use crate::Error;

const BOUNDARY: &str = "----boundary";

pub(crate) fn encode(parts: &[(&str, Option<&str>, &str, &str)]) -> (String, Vec<u8>) {
    let content_type = format!("multipart/form-data; boundary={BOUNDARY}");
    let mut body = String::new();

    for (name, filename, content_type, content) in parts {
        body.push_str(&format!("--{BOUNDARY}\r\n"));
        body.push_str(&format!("Content-Disposition: form-data; name=\"{name}\""));
        if let Some(filename) = filename {
            body.push_str(&format!("; filename=\"{filename}\""));
        }
        body.push_str("\r\n");
        body.push_str(&format!("Content-Type: {content_type}\r\n\r\n"));
        body.push_str(content);
        body.push_str("\r\n");
    }

    body.push_str(&format!("--{BOUNDARY}--\r\n"));
    (content_type, body.into_bytes())
}

pub(crate) fn decode(content_type: &str, body: &[u8]) -> Result<Vec<(String, String)>, Error> {
    if !content_type.starts_with("multipart/") {
        return Err(Error::Unexpected(format!("Expected multipart content, got {content_type:?}")));
    }

    let boundary = header_parameter(content_type, "boundary")
        .filter(|boundary| !boundary.is_empty())
        .ok_or_else(|| Error::Unexpected("Missing multipart boundary".into()))?;
    let opening_boundary = format!("--{boundary}\r\n");
    let part_separator = format!("\r\n--{boundary}");

    let text = String::from_utf8_lossy(body);
    let content = text
        .strip_prefix(&opening_boundary)
        .ok_or_else(|| Error::Unexpected("Invalid multipart content".into()))?;

    let mut parts = Vec::new();
    for part in content.split(&part_separator) {
        if part.starts_with("--") {
            break;
        }
        parts.push(decode_part(part)?);
    }
    Ok(parts)
}

fn decode_part(part: &str) -> Result<(String, String), Error> {
    let part = part.strip_prefix("\r\n").unwrap_or(part);
    let (headers, content) = part
        .split_once("\r\n\r\n")
        .ok_or_else(|| Error::Unexpected("Invalid multipart part".into()))?;

    let disposition = headers
        .split("\r\n")
        .filter_map(|line| line.split_once(':'))
        .find(|(name, _)| name.eq_ignore_ascii_case("content-disposition"))
        .map(|(_, value)| value)
        .ok_or_else(|| Error::Unexpected("Missing multipart content disposition".into()))?;

    let name = header_parameter(disposition, "name")
        .or_else(|| header_parameter(disposition, "filename"))
        .ok_or_else(|| Error::Unexpected("Missing multipart part name".into()))?;

    Ok((name.to_owned(), content.to_owned()))
}

fn header_parameter<'a>(header: &'a str, key: &str) -> Option<&'a str> {
    for parameter in header.split(';').skip(1) {
        let Some((name, value)) = parameter.trim().split_once('=') else {
            continue;
        };
        if name.eq_ignore_ascii_case(key) {
            return Some(value.trim().trim_matches('"'));
        }
    }
    None
}
