use anyhow::{Context, Result};
use giong_cloudflare::Api;
use giong_cloudflare_api::{CreateR2BucketParams, R2Api, R2Bucket};
use giong_provider::bindings::{enums::ChangeStatus, helpers::diff};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Config {
    name: String,
}

pub async fn read(input: &str) -> Result<ChangeStatus> {
    let config: Config = serde_json::from_str(input)?;
    let resource = Api::new().get_r2_bucket(&config.name).await?;
    Ok(diff(&config, &resource))
}

pub async fn apply(input: &str) -> Result<String> {
    let config: Config = serde_json::from_str(input)?;
    let resource = Api::new()
        .create_r2_bucket(&CreateR2BucketParams { name: config.name })
        .await?
        .context("Cloudflare API returned no R2 bucket")?;
    Ok(serde_json::to_string(&resource)?)
}

pub async fn delete(input: &str) -> Result<()> {
    let resource: R2Bucket = serde_json::from_str(input)?;
    Api::new().delete_r2_bucket(&resource.name).await?;
    Ok(())
}

giong_provider::export_resource!(R2Resource);

fn main() {}
