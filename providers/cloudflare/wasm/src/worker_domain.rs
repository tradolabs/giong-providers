use anyhow::{Context, Result};
use giong_cloudflare::Api;
use giong_cloudflare_api::{CreateWorkerDomainParams, WorkerDomain, WorkerDomainApi};
use giong_provider::bindings::{enums::ChangeStatus, helpers::diff};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Config {
    hostname: String,
    zone_name: String,
    service: String,
}

pub async fn read(input: &str) -> Result<ChangeStatus> {
    let config: Config = serde_json::from_str(input)?;
    let domains =
        Api::new().search_worker_domains(Some(&config.hostname), Some(&config.zone_name)).await?;
    Ok(diff(&config, &domains.first()))
}

pub async fn apply(input: &str) -> Result<String> {
    let config: Config = serde_json::from_str(input)?;
    let resource = Api::new()
        .create_worker_domain(&CreateWorkerDomainParams {
            hostname: config.hostname,
            zone_name: config.zone_name,
            service: config.service,
            enabled: true,
        })
        .await?
        .context("Cloudflare API returned no domain")?;
    Ok(serde_json::to_string(&resource)?)
}

pub async fn delete(input: &str) -> Result<()> {
    let resource: WorkerDomain = serde_json::from_str(input)?;
    Api::new().delete_worker_domain(&resource.id).await?;
    Ok(())
}

giong_provider::export_resource!(WorkerDomainResource);

fn main() {}
