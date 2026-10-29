#[path = "../api.rs"]
mod api;
#[path = "../macros.rs"]
mod macros;

use anyhow::{Context, Result};
use api::Api;
use giong_api_client::HttpMethod;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    hostname: String,
    zone_id: String,
    service: String,
}

#[derive(Deserialize, Serialize)]
struct State {
    id: String,
    hostname: String,
    zone_id: String,
    service: String,
}

pub fn read(input: &str) -> Result<Option<String>> {
    let config: Config = serde_json::from_str(input)?;
    let domains: Vec<State> = Api::request(HttpMethod::Get)
        .path("workers/domains")
        .query("hostname", &config.hostname)
        .query("zone_id", &config.zone_id)
        .send()?
        .context("Cloudflare API returned no domain list")?;
    domains
        .into_iter()
        .find(|domain| domain.hostname == config.hostname && domain.zone_id == config.zone_id)
        .map(|state| Ok(serde_json::to_string(&state)?))
        .transpose()
}

pub fn apply(input: &str, current: Option<&str>) -> Result<String> {
    let config: Config = serde_json::from_str(input)?;
    let current: Option<State> = current.map(serde_json::from_str).transpose()?;
    let state: State = Api::request(HttpMethod::Put)
        .path("workers/domains")
        .body(serde_json::to_vec(&json!({
            "hostname": config.hostname,
            "zone_id": config.zone_id,
            "service": config.service,
            "enabled": true,
        }))?)
        .send()?
        .context("Cloudflare API returned no domain")?;

    if let Some(current) = current.filter(|current| current.id != state.id) {
        delete(&serde_json::to_string(&current)?)?;
    }

    Ok(serde_json::to_string(&state)?)
}

pub fn delete(input: &str) -> Result<()> {
    let state: State = serde_json::from_str(input)?;
    Api::request(HttpMethod::Delete)
        .path(format!("workers/domains/{}", state.id))
        .send::<Value>()?;
    Ok(())
}

macros::export_resource!(WorkerDomainResource);

fn main() {}
