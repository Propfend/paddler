use std::{
    net::SocketAddr,
    os::unix::process::CommandExt,
    path::Path,
    process::{Child, Command, Stdio},
    str,
};

use async_trait::async_trait;
use log::{debug, error, info, warn};
use pingora::{server::ShutdownWatch, services::Service};
use tokio::{
    sync::broadcast::Receiver,
    time::{interval, Duration, MissedTickBehavior},
};

#[cfg(unix)]
use pingora::server::ListenFds;

use crate::errors::{app_error::AppError, result::Result};

pub struct ApplyingService {
    port: String,
    llama_path: String,
    model_path: String,
    monitoring_interval: Duration,
    llama_process: Option<Child>,
    update_model: Receiver<String>,
    update_binary: Receiver<String>,
    update_addr_rx: Receiver<String>,
}

impl ApplyingService {
    pub fn new(
        addr: SocketAddr,
        llama_path: String,
        model_path: String,
        monitoring_interval: Duration,
        update_model: Receiver<String>,
        update_binary: Receiver<String>,
        update_addr_rx: Receiver<String>,
    ) -> Result<Self> {
        let port = get_port(addr.to_string());
        Ok(ApplyingService {
            port,
            llama_path,
            model_path,
            monitoring_interval,
            llama_process: None,
            update_model,
            update_binary,
            update_addr_rx,
        })
    }

    async fn start_llamacpp_server(&mut self) -> Result<()> {
        unsafe {
            let mut cmd = Command::new(self.llama_path.to_owned());

            if !is_a_gguf_file(self.model_path.to_string()) {
                return Err(AppError::InvalidFileError(
                    "Insert a an existent gguf file for a model.".to_owned(),
                ));
            }

            cmd.args(&[
                "-m",
                self.model_path.as_str(),
                "--port",
                &self.port,
                "--slots",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null());

            #[cfg(unix)]
            cmd.pre_exec(|| {
                libc::setsid();

                Ok(())
            });

            let child = cmd.spawn()?;
            self.llama_process = Some(child);
        }

        Ok(())
    }

    fn server_is_running(&mut self) -> bool {
        if let Some(child) = &mut self.llama_process {
            match child.try_wait() {
                Ok(Some(_)) => false,
                Ok(None) => true,
                Err(e) => {
                    error!("Error checking process status: {}", e);
                    false
                }
            }
        } else {
            false
        }
    }
}

#[async_trait]
impl Service for ApplyingService {
    async fn start_service(
        &mut self,
        #[cfg(unix)] _fds: Option<ListenFds>,
        mut shutdown: ShutdownWatch,
    ) {
        let mut ticker = interval(self.monitoring_interval);
        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                _ = shutdown.changed() => {
                    debug!("Shutting down supervising service");
                    return;
                },
                _ = ticker.tick() => {
                    if !self.server_is_running() {
                        if let Err(e) = self.start_llamacpp_server().await {
                            error!("Failed to start llama server: {}", e);
                        }
                        info!("Llamacpp server fell off. Restarting server");
                    }
                },
                input_path = self.update_model.recv() => {
                    match input_path {
                        Ok(path) => {
                            self.model_path = path;
                            match self.start_llamacpp_server().await {
                                Ok(_) => {info!("Model Path was updated. Restarting server");},
                                Err(e) => {warn!("Failed to start llama server. Changes were not applied {}", e);}
                            }
                        },
                        Err(e) => {
                            error!("Failed to receive model path: {}", e);
                        }
                    }
                },
                input_path = self.update_binary.recv() => {
                    match input_path {
                        Ok(path) => {
                            self.llama_path = path;
                            match self.start_llamacpp_server().await {
                                Ok(_) => {info!("Binary path was updated. Restarting server");},
                                Err(e) => {warn!("Failed to start llama server. Changes were not applied {}", e);}
                            }
                        },
                        Err(e) => {
                            error!("Failed to receive binary path: {}", e);
                        }
                    }
                }
            }
        }
    }

    fn name(&self) -> &str {
        "applying"
    }

    fn threads(&self) -> Option<usize> {
        None
    }
}

fn get_port(addr: String) -> String {
    unsafe {
        addr.split(':')
            .nth(1)
            .unwrap_unchecked()
            .parse::<String>()
            .unwrap_unchecked()
    }
}

fn is_a_gguf_file(path: String) -> bool {
    let file = Path::new(&path);

    if file.exists() {
        if let Some(ext) = file.extension() {
            if ext.to_str() == Some("gguf") {
                return true;
            }

            return false;
        }
        return false;
    }

    false
}
