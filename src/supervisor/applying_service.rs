use std::{
    net::SocketAddr,
    os::unix::process::CommandExt,
    process::{Child, Command, Stdio},
};

use async_trait::async_trait;
use log::warn;
use log::{debug, error};
use pingora::{server::ShutdownWatch, services::Service};
use tokio::time::{interval, Duration, MissedTickBehavior};

#[cfg(unix)]
use pingora::server::ListenFds;

use crate::errors::result::Result;

pub struct ApplyingService {
    port: String,
    llama_server_path: String,
    model_path: String,
    monitoring_interval: Duration,
    llama_process: Option<Child>,
}

impl ApplyingService {
    pub fn new(
        addr: SocketAddr,
        llama_server_path: String,
        model_path: String,
        monitoring_interval: Duration,
    ) -> Result<Self> {
        let port = get_port(addr.to_string());
        Ok(ApplyingService {
            port,
            llama_server_path,
            model_path,
            monitoring_interval,
            llama_process: None,
        })
    }

    async fn start_llamacpp_server(&mut self) -> Result<()> {
        unsafe {
            let mut cmd = Command::new(self.llama_server_path.to_owned());
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
                        warn!("Llamacpp server fell off. Restarting server");
                    };
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
