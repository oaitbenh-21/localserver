// src/request.rs

use std::collections::HashMap;

#[derive(Debug)]
pub enum Method {
    Get,
    Post,
    Delete,
    Unknown(String),
}

#[derive(Debug)]
pub struct Request {
    pub method: Method,
    pub path: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Request {
    pub fn parse(buffer: &[u8]) -> Option<Request> {
        // Split headers and body on the blank line
        let separator = b"\r\n\r\n";
        let header_end = buffer.windows(4).position(|w| w == separator)?;

        let header_section = &buffer[..header_end];
        let body = buffer[header_end + 4..].to_vec();

        // Parse the header section as text
        let header_text = String::from_utf8_lossy(header_section);
        let mut lines = header_text.lines();

        // First line is the request line
        let first_line = lines.next()?;
        let mut parts = first_line.split_whitespace();

        let method = match parts.next()? {
            "GET" => Method::Get,
            "POST" => Method::Post,
            "DELETE" => Method::Delete,
            other => Method::Unknown(other.to_string()),
        };

        let path = parts.next()?.to_string();
        let version = parts.next()?.to_string();

        // Remaining lines are headers — "Key: Value"
        let mut headers = HashMap::new();
        for line in lines {
            if let Some((key, value)) = line.split_once(':') {
                headers.insert(key.trim().to_lowercase(), value.trim().to_string());
            }
        }

        Some(Request {
            method,
            path,
            version,
            headers,
            body,
        })
    }

    pub fn content_length(&self) -> usize {
        self.headers
            .get("content-length")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // ── Valid requests ────────────────────────────────────────────────────

    #[test]
    fn test_parse_get_root() {
        let raw = b"GET / HTTP/1.1\r\nHost: localhost:8080\r\n\r\n";
        let req = Request::parse(raw).expect("should parse");

        assert!(matches!(req.method, Method::Get));
        assert_eq!(req.path, "/");
        assert_eq!(req.version, "HTTP/1.1");
    }

    #[test]
    fn test_parse_get_with_path() {
        let raw = b"GET /about.html HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let req = Request::parse(raw).expect("should parse");

        assert!(matches!(req.method, Method::Get));
        assert_eq!(req.path, "/about.html");
    }

    #[test]
    fn test_parse_post() {
        let body = "hello world";
        let raw = format!(
            "POST /upload HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );
        let req = Request::parse(raw.as_bytes()).expect("should parse");

        assert!(matches!(req.method, Method::Post));
        assert_eq!(req.path, "/upload");
        assert_eq!(req.body, body.as_bytes());
    }

    #[test]
    fn test_parse_delete() {
        let raw = b"DELETE /file.txt HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let req = Request::parse(raw).expect("should parse");

        assert!(matches!(req.method, Method::Delete));
        assert_eq!(req.path, "/file.txt");
    }

        #[test]
    fn test_parse_unknown_method() {
        let raw = b"PATCH /file.txt HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let req = Request::parse(raw).expect("should parse");

        assert!(matches!(req.method, Method::Unknown(_)));
        if let Method::Unknown(m) = req.method {
            assert_eq!(m, "PATCH");
        }
    }
}
