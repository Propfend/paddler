use actix_web::web::Bytes;
use async_trait::async_trait;
use log::{debug, error, info};
use pingora::server::ShutdownWatch;
use pingora::services::Service;
use tokio::sync::broadcast::Sender;
use tokio::time::{interval, Duration, MissedTickBehavior};
use tokio_stream::wrappers::BroadcastStream;
use url::Url;
use uuid::Uuid;

#[cfg(unix)]
use pingora::server::ListenFds;

use crate::errors::result::Result;

pub struct ReportingService {
    stats_endpoint_url: String,
    status_update_tx: Sender<Bytes>,
}

impl ReportingService {
    pub fn new(management_addr: Url, status_update_tx: Sender<Bytes>) -> Result<Self> {
        let agent_id = Uuid::new_v4();

        Ok(ReportingService {
            stats_endpoint_url: management_addr
                .join(&format!("/status_update/{}", agent_id))?
                .to_string(),
            status_update_tx,
        })
    }

    async fn keep_connection_alive(&self) -> Result<()> {
        let status_update_rx = self.status_update_tx.subscribe();
        let stream = BroadcastStream::new(status_update_rx);
        let reqwest_body = reqwest::Body::wrap_stream(stream);

        info!("Establishing connection with management server");

        match reqwest::Client::new()
            .post(self.stats_endpoint_url.clone())
            .body(reqwest_body)
            .send()
            .await
        {
            Ok(_) => {
                error!("Management server connection closed");

                Ok(())
            }
            Err(err) => Err(err.into()),
        }
    }
}

#[async_trait]
impl Service for ReportingService {
    async fn start_service(
        &mut self,
        #[cfg(unix)] _fds: Option<ListenFds>,
        mut shutdown: ShutdownWatch,
    ) {
        let mut ticker = interval(Duration::from_secs(1));

        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                _ = shutdown.changed() => {
                    debug!("Shutting down reporting service");
                    return;
                },
                _ = ticker.tick() => {
                    if let Err(err) = self.keep_connection_alive().await {
                        error!("Failed to keep the connection alive: {}", err);
                    }
                }
            }
        }
    }

    fn name(&self) -> &str {
        "reporting"
    }

    fn threads(&self) -> Option<usize> {
        None
    }
}