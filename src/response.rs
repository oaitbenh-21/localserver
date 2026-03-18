// src/response.rs

use std::io::Write;
use std::net::TcpStream;

pub enum StatusCode {
    Ok,
    BadRequest,
    Forbidden,
    NotFound,
    MethodNotAllowed,
    ContentTooLarge,
    InternalServerError,
}

impl StatusCode {
    fn as_str(&self) -> &str {
        match self {
            StatusCode::Ok => "200 OK",
            StatusCode::BadRequest => "400 Bad Request",
            StatusCode::Forbidden => "403 Forbidden",
            StatusCode::NotFound => "404 Not Found",
            StatusCode::MethodNotAllowed => "405 Method Not Allowed",
            StatusCode::ContentTooLarge => "413 Content Too Large",
            StatusCode::InternalServerError => "500 Internal Server Error",
        }
    }
}

pub struct Response {
    pub status: StatusCode,
    pub content_type: String,
    pub body: Vec<u8>,
}

impl Response {
    pub fn new(status: StatusCode, content_type: &str, body: Vec<u8>) -> Response {
        Response {
            status,
            content_type: content_type.to_string(),
            body,
        }
    }

    pub fn send(&self, stream: &mut TcpStream) {
        let header = format!(
            "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
            self.status.as_str(),
            self.content_type,
            self.body.len()
        );
        // write header
        if let Err(e) = stream.write_all(header.as_bytes()) {
            eprintln!("Failed to write response header: {}", e);
            return;
        }
        // write body
        if let Err(e) = stream.write_all(&self.body) {
            eprintln!("Failed to write response body: {}", e);
        }
    }

    // Convenience constructors for common error responses
    pub fn error(status: StatusCode) -> Response {
        let body = format!("<html><body><h1>{}</h1></body></html>", status.as_str());
        Response::new(status, "text/html", body.into_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use std::net::{TcpListener, TcpStream};

    // ── Helper ────────────────────────────────────────────────────────────
    // Sends a response to a real TcpStream and reads back the raw bytes
    // This lets us test the actual bytes that go over the wire
    // in darija this is just making a socket and reading from it huh, prettey stupidly elegant.
    fn capture_response(response: Response) -> Vec<u8> {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let handle = std::thread::spawn(move || {
            let mut client = TcpStream::connect(addr).unwrap();
            let mut buf = Vec::new();
            client.read_to_end(&mut buf).unwrap();
            buf
        });

        let (mut stream, _) = listener.accept().unwrap();
        response.send(&mut stream);
        drop(stream); // close connection so client's read_to_end finishes

        handle.join().unwrap()
    }
    // ── Wire format ───────────────────────────────────────────────────────

    #[test]
    fn test_response_starts_with_http_version() {
        let res = Response::new(StatusCode::Ok, "text/plain", b"hi".to_vec());
        let bytes = capture_response(res);
        let raw = String::from_utf8_lossy(&bytes);

        assert!(raw.starts_with("HTTP/1.1"));
    }

    #[test]
    fn test_response_contains_status_line() {
        let res = Response::new(StatusCode::Ok, "text/plain", b"hi".to_vec());
        let bytes = capture_response(res);
        let raw = String::from_utf8_lossy(&bytes);

        assert!(raw.contains("200 OK"));
    }

    #[test]
    fn test_response_contains_content_type_header() {
        let res = Response::new(StatusCode::Ok, "text/css", b"body{}".to_vec());
        let bytes = capture_response(res);
        let raw = String::from_utf8_lossy(&bytes);

        assert!(raw.contains("Content-Type: text/css"));
    }

    #[test]
    fn test_content_length_matches_body() {
        let body = b"hello world".to_vec();
        let res = Response::new(StatusCode::Ok, "text/plain", body);
        let bytes = capture_response(res);
        let raw = String::from_utf8_lossy(&bytes);

        assert!(raw.contains("Content-Length: 11"));
    }

    #[test]
    fn test_header_and_body_separated_by_blank_line() {
        let res = Response::new(StatusCode::Ok, "text/plain", b"body".to_vec());
        let bytes = capture_response(res);
        let raw = String::from_utf8_lossy(&bytes);

        // \r\n\r\n must exist between headers and body
        assert!(raw.contains("\r\n\r\n"));
    }

    #[test]
    fn test_body_is_after_separator() {
        let res = Response::new(StatusCode::Ok, "text/plain", b"hello".to_vec());
        let bytes = capture_response(res);
        let raw = String::from_utf8_lossy(&bytes);

        let separator = "\r\n\r\n";
        let body_start = raw.find(separator).unwrap() + separator.len();
        assert_eq!(&raw[body_start..], "hello");
    }

    #[test]
    fn test_binary_body_survives_wire() {
        let body = vec![0xFF, 0xFE, 0x00, 0x01];
        let res = Response::new(StatusCode::Ok, "application/octet-stream", body.clone());
        let bytes = capture_response(res);

        // Find where body starts after \r\n\r\n
        let separator = b"\r\n\r\n";
        let sep_pos = bytes.windows(4).position(|w| w == separator).unwrap();
        let actual_body = &bytes[sep_pos + 4..];

        assert_eq!(actual_body, body.as_slice());
    }
}
