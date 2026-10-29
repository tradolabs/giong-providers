use serde::de::DeserializeOwned;
use serde_json::Value;
use url::Url;

pub mod error;

pub use error::Error;

pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

pub struct HttpResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

pub type Transport = fn(HttpRequest) -> Result<HttpResponse, Error>;

#[cfg(not(target_arch = "wasm32"))]
fn default_transport(request: HttpRequest) -> Result<HttpResponse, Error> {
    let method = reqwest::Method::from_bytes(request.method.as_bytes())
        .map_err(|error| Error::Unexpected(error.to_string()))?;
    let mut builder = reqwest::blocking::Client::new().request(method, &request.url);
    for (name, value) in request.headers {
        builder = builder.header(name, value);
    }
    let response = builder.body(request.body).send()?;
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
    let body = response.bytes()?.to_vec();
    Ok(HttpResponse { status, headers, body })
}

#[cfg(target_arch = "wasm32")]
fn default_transport(_request: HttpRequest) -> Result<HttpResponse, Error> {
    Err(Error::Unexpected("A custom transport is required on wasm32".into()))
}

#[derive(Clone, Copy)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

impl HttpMethod {
    fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Patch => "PATCH",
            Self::Delete => "DELETE",
        }
    }
}

pub type ResponseParser = fn(HttpMethod, HttpResponse) -> Result<Option<Value>, Error>;

fn default_parser(method: HttpMethod, response: HttpResponse) -> Result<Option<Value>, Error> {
    if response.status == 404 && matches!(method, HttpMethod::Get | HttpMethod::Delete) {
        return Ok(None);
    }
    if !(200..300).contains(&response.status) {
        return Err(Error::Http(format!(
            "HTTP {}: {}",
            response.status,
            String::from_utf8_lossy(&response.body)
        )));
    }
    if response.status == 204 || response.body.is_empty() {
        return Ok(Some(Value::Null));
    }
    Ok(Some(serde_json::from_slice(&response.body)?))
}

pub struct HttpRequestBuilder {
    transport: Transport,
    method: HttpMethod,
    base_url: String,
    path: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
    response_parser: ResponseParser,
}

impl HttpRequestBuilder {
    pub fn new(method: HttpMethod) -> Self {
        Self {
            transport: default_transport,
            method,
            base_url: String::new(),
            path: String::new(),
            headers: Vec::new(),
            body: Vec::new(),
            response_parser: default_parser,
        }
    }

    pub fn transport(mut self, transport: Transport) -> Self {
        self.transport = transport;
        self
    }

    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = path.into();
        self
    }

    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }

    pub fn response_parser(mut self, parser: ResponseParser) -> Self {
        self.response_parser = parser;
        self
    }

    pub fn multipart(self, parts: &[(&str, Option<&str>, &str, &str)]) -> Self {
        let boundary = "----boundary";
        let mut body = String::new();
        for (name, filename, content_type, content) in parts {
            body.push_str(&format!(
                "--{boundary}\r\nContent-Disposition: form-data; name=\"{name}\""
            ));
            if let Some(filename) = filename {
                body.push_str(&format!("; filename=\"{filename}\""));
            }
            body.push_str(&format!("\r\nContent-Type: {content_type}\r\n\r\n{content}\r\n"));
        }
        body.push_str(&format!("--{boundary}--\r\n"));
        self.header("content-type", format!("multipart/form-data; boundary={boundary}"))
            .body(body.into_bytes())
    }

    pub fn send<T: DeserializeOwned>(mut self) -> Result<Option<T>, Error> {
        let base = self.base_url.trim_end_matches('/');
        let path = self.path.trim_start_matches('/');
        let url = Url::parse(&format!("{base}/{path}"))?;
        if !self.headers.iter().any(|(k, _)| k.eq_ignore_ascii_case("content-type")) {
            self.headers.push(("content-type".into(), "application/json".into()));
        }
        let response = (self.transport)(HttpRequest {
            method: self.method.as_str().into(),
            url: url.into(),
            headers: self.headers,
            body: self.body,
        })?;
        let value = (self.response_parser)(self.method, response)?;
        value.map(|v| Ok(serde_json::from_value(v)?)).transpose()
    }
}
