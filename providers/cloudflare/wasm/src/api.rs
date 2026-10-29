use crate::giong::provider::host;
use giong_api_client::{Error, HttpMethod, HttpRequest, HttpRequestBuilder, HttpResponse};
use serde_json::Value;
use std::sync::OnceLock;

pub struct Api {
    account_id: String,
    api_token: String,
}

static INSTANCE: OnceLock<Api> = OnceLock::new();

impl Api {
    pub fn instance() -> &'static Self {
        INSTANCE.get_or_init(|| Self {
            account_id: host::secret("CLOUDFLARE_ACCOUNT_ID")
                .expect("CLOUDFLARE_ACCOUNT_ID is required"),
            api_token: host::secret("CLOUDFLARE_API_TOKEN")
                .expect("CLOUDFLARE_API_TOKEN is required"),
        })
    }

    pub fn request(method: HttpMethod) -> HttpRequestBuilder {
        let api = Self::instance();
        HttpRequestBuilder::new(method)
            .transport(wasm_http)
            .base_url(format!("https://api.cloudflare.com/client/v4/accounts/{}", api.account_id))
            .header("authorization", format!("Bearer {}", api.api_token))
            .response_parser(parse_response)
    }
}

fn wasm_http(request: HttpRequest) -> Result<HttpResponse, Error> {
    let response = host::http(&host::HttpRequest {
        method: request.method,
        url: request.url,
        headers: request.headers,
        body: request.body,
    })
    .map_err(Error::Unexpected)?;
    Ok(HttpResponse { status: response.status, headers: response.headers, body: response.body })
}

fn parse_response(method: HttpMethod, response: HttpResponse) -> Result<Option<Value>, Error> {
    let status = response.status;

    if status == 404 && matches!(method, HttpMethod::Get | HttpMethod::Delete) {
        return Ok(None);
    }
    if status == 204 {
        return Ok(Some(Value::Null));
    }

    let mut envelope: Value = serde_json::from_slice(&response.body).unwrap_or(Value::Null);

    if !(200..300).contains(&status) || envelope["success"] != true {
        return Err(Error::Http(format!(
            "Cloudflare API returned HTTP {status}: {}",
            envelope["errors"]
        )));
    }

    Ok(Some(envelope["result"].take()))
}
