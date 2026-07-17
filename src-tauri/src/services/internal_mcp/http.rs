use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const MAX_HEADER_BYTES: usize = 16 * 1024;
pub(crate) const MAX_BODY_BYTES: usize = 1024 * 1024;

pub struct HttpRequest {
    pub authorization: String,
    pub body: Vec<u8>,
}

pub(crate) struct RequestHead {
    end: usize,
    length: usize,
    authorization: String,
}

pub(crate) fn parse_head(data: &[u8]) -> Result<RequestHead, String> {
    let end = data
        .windows(4)
        .position(|part| part == b"\r\n\r\n")
        .map(|position| position + 4)
        .ok_or_else(generic_error)?;
    if end > MAX_HEADER_BYTES {
        return Err(generic_error());
    }
    let text = std::str::from_utf8(&data[..end]).map_err(|_| generic_error())?;
    let mut lines = text.split("\r\n");
    if lines.next() != Some("POST /mcp HTTP/1.1") {
        return Err(generic_error());
    }
    let mut length = None;
    let mut authorization = None;
    for line in lines.filter(|line| !line.is_empty()) {
        let (name, value) = line.split_once(':').ok_or_else(generic_error)?;
        let value = value.trim();
        match name.to_ascii_lowercase().as_str() {
            "content-length" if length.is_none() => {
                length = Some(value.parse::<usize>().map_err(|_| generic_error())?);
            }
            "authorization" if authorization.is_none() && value.len() <= 256 => {
                authorization = Some(value.to_string());
            }
            "transfer-encoding" => return Err(generic_error()),
            _ => {}
        }
    }
    let length = length.ok_or_else(generic_error)?;
    if length > MAX_BODY_BYTES {
        return Err(generic_error());
    }
    Ok(RequestHead {
        end,
        length,
        authorization: authorization.ok_or_else(generic_error)?,
    })
}

pub async fn read_request(stream: &mut TcpStream) -> Result<HttpRequest, String> {
    let mut data = Vec::with_capacity(8192);
    let head = loop {
        if let Ok(head) = parse_head(&data) {
            break head;
        }
        if data.len() >= MAX_HEADER_BYTES {
            return Err(generic_error());
        }
        let mut chunk = [0_u8; 4096];
        let count = stream.read(&mut chunk).await.map_err(|_| generic_error())?;
        if count == 0 {
            return Err(generic_error());
        }
        data.extend_from_slice(&chunk[..count]);
    };
    let total = head.end.saturating_add(head.length);
    while data.len() < total {
        let mut chunk = [0_u8; 8192];
        let count = stream.read(&mut chunk).await.map_err(|_| generic_error())?;
        if count == 0 {
            return Err(generic_error());
        }
        if data.len().saturating_add(count) > MAX_HEADER_BYTES + MAX_BODY_BYTES {
            return Err(generic_error());
        }
        data.extend_from_slice(&chunk[..count]);
    }
    Ok(HttpRequest {
        authorization: head.authorization,
        body: data[head.end..total].to_vec(),
    })
}

pub async fn write_response(
    stream: &mut TcpStream,
    status: &str,
    body: Option<&serde_json::Value>,
) -> Result<(), String> {
    let payload = body
        .map(serde_json::to_vec)
        .transpose()
        .map_err(|_| generic_error())?
        .unwrap_or_default();
    let content_type = if body.is_some() {
        "Content-Type: application/json\r\n"
    } else {
        ""
    };
    let header = format!(
        "HTTP/1.1 {status}\r\n{content_type}Content-Length: {}\r\nConnection: close\r\n\r\n",
        payload.len()
    );
    stream
        .write_all(header.as_bytes())
        .await
        .map_err(|_| generic_error())?;
    stream
        .write_all(&payload)
        .await
        .map_err(|_| generic_error())
}

fn generic_error() -> String {
    "Requête MCP invalide".to_string()
}
