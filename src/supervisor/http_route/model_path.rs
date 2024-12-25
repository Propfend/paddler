use actix_web::{
    get,
    web::{self, Bytes},
    HttpResponse,
};
use tokio::sync::broadcast::Sender;

use crate::errors::result::Result;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(respond);
}

#[get("llamacpp/model/{path:.*}")]
async fn respond(
    status_update_tx: web::Data<Sender<Bytes>>,
    path: web::Path<(String)>,
) -> Result<HttpResponse> {
    let model_path = path.into_inner();

    status_update_tx.send(Bytes::from(model_path))?;

    Ok(HttpResponse::Ok().finish())
}