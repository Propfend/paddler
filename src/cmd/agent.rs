use actix_web::web::Bytes;
use pingora::server::{configuration::Opt, Server};
use std::{net::SocketAddr, time::Duration};
use tokio::sync::broadcast::channel;

use crate::agent::monitoring_service::MonitoringService;
use crate::agent::reporting_service::ReportingService;
use crate::errors::result::Result;
use crate::llamacpp::llamacpp_client::LlamacppClient;
use crate::BackendDriver;

pub fn handle(
    management_addr: SocketAddr,
    monitoring_interval: Duration,
    backend_driver: BackendDriver,
) -> Result<()> {
    let (status_update_tx, _status_update_rx) = channel::<Bytes>(1);

    let llamacpp_client = LlamacppClient::new(backend_driver.clone())?;

    let monitoring_service = MonitoringService::new(
        backend_driver,
        llamacpp_client,
        monitoring_interval,
        status_update_tx.clone(),
    )?;

    let reporting_service = ReportingService::new(management_addr, status_update_tx)?;

    let mut pingora_server = Server::new(Opt {
        upgrade: false,
        daemon: false,
        nocapture: false,
        test: false,
        conf: None,
    })?;

    pingora_server.bootstrap();
    pingora_server.add_service(monitoring_service);
    pingora_server.add_service(reporting_service);
    pingora_server.run_forever();
}
