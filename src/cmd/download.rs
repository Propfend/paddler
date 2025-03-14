use std::path::PathBuf;

use hf_hub::api::sync::ApiBuilder;
use log::{error, info};

use crate::errors::result::Result;

pub fn handle(
    model_name: String,
    filename: String,
    token: Option<String>,
    path: Option<PathBuf>,
) -> Result<()> {
    let api = match path {
        Some(path) => ApiBuilder::new()
            .with_token(token)
            .with_cache_dir(path)
            .with_progress(true)
            .build()?,
        None => ApiBuilder::new()
            .with_token(token)
            .with_progress(true)
            .build()?,
    };

    let repo = api.model(model_name.to_string());
    match repo.get(&filename) {
        Ok(filename) => info!("Model successfully downloaded in: {:#?}", filename),
        Err(_) => error!("Error while downloading model. Wrong token, file or model name"),
    };

    Ok(())
}
