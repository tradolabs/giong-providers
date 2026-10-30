use crate::{Error, HttpRequest, HttpResponse};

pub type Transport = fn(HttpRequest) -> Result<HttpResponse, Error>;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) async fn default_http(request: HttpRequest) -> Result<HttpResponse, Error> {
    static CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();

    let method = reqwest::Method::from_bytes(request.method.as_bytes())
        .map_err(|error| Error::Unexpected(error.to_string()))?;
    let mut builder = CLIENT.get_or_init(reqwest::Client::new).request(method, &request.url);
    for (name, value) in request.headers {
        builder = builder.header(name, value);
    }
    let response = builder.body(request.body).send().await?;
    let status = response.status().as_u16();
    let headers = response
        .headers()
        .iter()
        .map(|(name, value)| {
            value
                .to_str()
                .map(|value| (name.to_string(), value.to_owned()))
                .map_err(|error| Error::Unexpected(error.to_string()))
        })
        .collect::<Result<Vec<_>, Error>>()?;
    let body = response.bytes().await?.to_vec();
    Ok(HttpResponse { status, headers, body })
}

#[cfg(target_arch = "wasm32")]
pub(crate) async fn default_http(_request: HttpRequest) -> Result<HttpResponse, Error> {
    Err(Error::Unexpected("A custom transport is required on wasm32".into()))
}
