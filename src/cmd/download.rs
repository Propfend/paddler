use hf_hub::api::sync::Api;
use log::{info, error};

use crate::errors::result::Result;

pub fn handle(
    model_name: String,
) -> Result<()> {
    let api = Api::new()?;

    let repo = api.model(model_name);

    match repo.get("config.json") {
        Ok(path) => {
            info!("Model downloaded successufully in {:#?}", path);
        }
        Err(err) => error!("Error while downloading model: {:#?}", err),
    };

    Ok(())
}
