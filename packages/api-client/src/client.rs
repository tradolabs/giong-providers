use crate::transport::default_http;
use crate::{Error, HttpMethod, HttpRequest, HttpResponse, ResponseParser, Transport, multipart};
use serde::{Serialize, de::DeserializeOwned};
use url::Url;

pub struct HttpRequestBuilder {
    transport: Option<Transport>,
    method: HttpMethod,
    base_url: String,
    path: String,
    query: Vec<(String, String)>,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
    response_parser: ResponseParser,
}

impl HttpRequestBuilder {
    pub fn new(method: HttpMethod) -> Self {
        Self {
            transport: None,
            method,
            base_url: String::new(),
            path: String::new(),
            query: Vec::new(),
            headers: Vec::new(),
            body: Vec::new(),
            response_parser: HttpResponse::parse,
        }
    }

    pub fn transport(mut self, transport: Transport) -> Self {
        self.transport = Some(transport);
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

    pub fn query(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.query.push((name.into(), value.into()));
        self
    }

    pub fn body(mut self, body: &(impl Serialize + ?Sized)) -> Result<Self, Error> {
        self.body = serde_json::to_vec(body)?;
        Ok(self)
    }

    pub fn response_parser(mut self, parser: ResponseParser) -> Self {
        self.response_parser = parser;
        self
    }

    pub fn multipart(mut self, parts: &[(&str, Option<&str>, &str, &str)]) -> Self {
        let (content_type, body) = multipart::encode(parts);
        self.body = body;
        self.header("content-type", content_type)
    }

    pub async fn execute<T: DeserializeOwned>(mut self) -> Result<Option<T>, Error> {
        let base = self.base_url.trim_end_matches('/');
        let path = self.path.trim_start_matches('/');
        let mut url = Url::parse(&format!("{base}/{path}"))?;
        if !self.query.is_empty() {
            url.query_pairs_mut().extend_pairs(&self.query);
        }
        if !self.headers.iter().any(|(k, _)| k.eq_ignore_ascii_case("content-type")) {
            self.headers.push(("content-type".into(), "application/json".into()));
        }
        let request = HttpRequest {
            method: self.method.as_str().into(),
            url: url.into(),
            headers: self.headers,
            body: self.body,
        };
        let response = match self.transport {
            Some(transport) => transport(request)?,
            None => default_http(request).await?,
        };
        (self.response_parser)(self.method, response)?
            .map(serde_json::from_value)
            .transpose()
            .map_err(Error::from)
    }
}
