use giong_api_client::{Error as ClientError, HttpResponse};
use giong_cloudflare_api::CloudflareApi;
use giong_provider::bindings::giong::provider::host;
use std::ops::Deref;

pub struct Api(CloudflareApi);

impl Api {
    pub fn new() -> Self {
        Self(CloudflareApi {
            account_id: host::secret("CLOUDFLARE_ACCOUNT_ID")
                .expect("CLOUDFLARE_ACCOUNT_ID is required"),
            api_token: host::secret("CLOUDFLARE_API_TOKEN")
                .expect("CLOUDFLARE_API_TOKEN is required"),
            transport: Some(|request| {
                let response = host::http(&host::HttpRequest {
                    method: request.method,
                    url: request.url,
                    headers: request.headers,
                    body: request.body,
                })
                .map_err(ClientError::Unexpected)?;
                Ok(HttpResponse {
                    status: response.status,
                    headers: response.headers,
                    body: response.body,
                })
            }),
        })
    }
}

impl Default for Api {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for Api {
    type Target = CloudflareApi;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
