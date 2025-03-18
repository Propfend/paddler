use actix_web::web::Bytes;
use async_trait::async_trait;
use log::{debug, error};
use pingora::{server::ShutdownWatch, services::Service};
use std::net::SocketAddr;
use tokio::{
    sync::broadcast::Sender,
    time::{interval, Duration, MissedTickBehavior},
};

#[cfg(unix)]
use pingora::server::ListenFds;

use crate::{
    balancer::status_update::StatusUpdate, errors::result::Result,
    llamacpp::llamacpp_client::LlamacppClient, BackendDriver,
};

pub struct MonitoringService {
    external_backend_addr: SocketAddr,
    llamacpp_client: LlamacppClient,
    monitoring_interval: Duration,
    name: Option<String>,
    status_update_tx: Sender<Bytes>,
}

impl MonitoringService {
    pub fn new(
        backend_driver: BackendDriver,
        llamacpp_client: LlamacppClient,
        monitoring_interval: Duration,
        status_update_tx: Sender<Bytes>,
    ) -> Result<Self> {
        let (name, external_backend_addr) = match backend_driver {
            BackendDriver::Llamacpp {
                external_llamacpp_addr,
                local_llamacpp_addr,
                llamacpp_api_key: _,
                name,
            } => {
                let addr = match external_llamacpp_addr {
                    Some(addr) => addr,
                    None => local_llamacpp_addr,
                };

                (name, addr)
            }
            BackendDriver::Ollama {
                external_ollama_addr,
                local_ollama_addr,
                ollama_api_key: _,
                max_slots: _,
                name,
            } => {
                let addr = match external_ollama_addr {
                    Some(addr) => addr,
                    None => local_ollama_addr,
                };

                (name, addr)
            }
        };

        Ok(MonitoringService {
            external_backend_addr,
            llamacpp_client,
            monitoring_interval,
            name,
            status_update_tx,
        })
    }

    async fn fetch_status(&self) -> Result<StatusUpdate> {
        match self.llamacpp_client.get_available_slots().await {
            Ok(slots_response) => Ok(StatusUpdate::new(
                self.name.to_owned(),
                None,
                self.external_backend_addr.to_owned(),
                slots_response.is_authorized,
                slots_response.is_slot_endpoint_enabled,
                slots_response.slots,
            )),
            Err(err) => Ok(StatusUpdate::new(
                self.name.to_owned(),
                Some(err.to_string()),
                self.external_backend_addr.to_owned(),
                None,
                None,
                vec![],
            )),
        }
    }

    async fn report_status(&self, status: StatusUpdate) -> Result<usize> {
        let status = Bytes::from(serde_json::to_vec(&status)?);

        Ok(self.status_update_tx.send(status)?)
    }
}

#[async_trait]
impl Service for MonitoringService {
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
                    debug!("Shutting down monitoring service");
                    return;
                },
                _ = ticker.tick() => {
                    match self.fetch_status().await {
                        Ok(status) => {
                            if let Err(err) = self.report_status(status).await {
                                error!("Failed to report status: {}", err);
                            }
                        }
                        Err(err) => {
                            error!("Failed to fetch status: {}", err);
                        }
                    }
                }
            }
        }
    }

    fn name(&self) -> &str {
        "monitoring"
    }

    fn threads(&self) -> Option<usize> {
        Some(1)
    }
}
