use crate::giong::provider::host::{self, HttpRequest};
use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use serde_json::Value;
use url::Url;

pub struct Api {
    account_id: String,
    api_token: String,
}

impl Api {
    pub fn new() -> Result<Self> {
        Ok(Self {
            account_id: host::secret("CLOUDFLARE_ACCOUNT_ID")
                .context("CLOUDFLARE_ACCOUNT_ID is required")?,
            api_token: host::secret("CLOUDFLARE_API_TOKEN")
                .context("CLOUDFLARE_API_TOKEN is required")?,
        })
    }

    pub fn request<T: DeserializeOwned>(
        &self,
        method: &str,
        path: &[&str],
        body: Vec<u8>,
    ) -> Result<Option<T>> {
        let mut url = Url::parse("https://api.cloudflare.com/client/v4/")?;
        url.path_segments_mut()
            .map_err(|_| anyhow::anyhow!("Invalid API base URL"))?
            .pop_if_empty()
            .extend(["accounts", &self.account_id])
            .extend(path);
        let response = host::http(&HttpRequest {
            method: method.into(),
            url: url.into(),
            headers: vec![
                ("authorization".into(), format!("Bearer {}", self.api_token)),
                ("content-type".into(), "application/json".into()),
            ],
            body,
        })
        .map_err(anyhow::Error::msg)?;
        if response.status == 404 {
            return Ok(None);
        }
        anyhow::ensure!(
            (200..300).contains(&response.status),
            "Cloudflare API returned HTTP {}",
            response.status
        );
        let mut envelope: Value = serde_json::from_slice(&response.body)?;
        anyhow::ensure!(envelope["success"] == true, "Cloudflare API request failed");
        Ok(Some(serde_json::from_value(envelope["result"].take())?))
    }
}
