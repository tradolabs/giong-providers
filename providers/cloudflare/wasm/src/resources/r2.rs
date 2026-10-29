#[path = "../api.rs"]
mod api;
#[path = "../macros.rs"]
mod macros;

use crate::api::Api;
use anyhow::{Context, Result};
use giong_api_client::HttpMethod;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct R2Config {
    pub name: String,
}

#[derive(Deserialize, Serialize)]
pub struct R2State {
    pub name: String,
    pub location: Option<String>,
    pub creation_date: Option<String>,
    pub storage_class: Option<String>,
}

pub fn read(config: &str) -> Result<Option<String>> {
    let config: R2Config = serde_json::from_str(config)?;
    let state: Option<R2State> = Api::request(HttpMethod::Get)
        .path(format!("r2/buckets/{}", config.name))
        .send()?;
    state.map(|state| Ok(serde_json::to_string(&state)?)).transpose()
}

pub fn apply(config: &str, current: Option<&str>) -> Result<String> {
    let config: R2Config = serde_json::from_str(config)?;
    let current: Option<R2State> = current.map(serde_json::from_str).transpose()?;
    if let Some(current) = current.filter(|state| state.name == config.name) {
        return Ok(serde_json::to_string(&current)?);
    }
    let state: R2State = Api::request(HttpMethod::Post)
        .path("r2/buckets")
        .body(serde_json::to_vec(&config)?)
        .send()?
        .context("Cloudflare API returned no resource")?;
    Ok(serde_json::to_string(&state)?)
}

pub fn delete(state: &str) -> Result<()> {
    let state: R2State = serde_json::from_str(state)?;
    Api::request(HttpMethod::Delete)
        .path(format!("r2/buckets/{}", state.name))
        .send::<serde_json::Value>()?;
    Ok(())
}

macros::export_resource!(R2Resource);

fn main() {}
