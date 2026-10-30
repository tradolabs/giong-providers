use crate::{Error, HttpMethod, multipart};
use serde_json::Value;

pub struct HttpResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }

    pub fn multipart(&self) -> Result<Vec<(String, String)>, Error> {
        multipart::decode(self.header("content-type").unwrap_or_default(), &self.body)
    }

    pub(crate) fn parse(
        method: HttpMethod,
        response: HttpResponse,
    ) -> Result<Option<Value>, Error> {
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
}

pub type ResponseParser = fn(HttpMethod, HttpResponse) -> Result<Option<Value>, Error>;
