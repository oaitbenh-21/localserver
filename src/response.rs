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
}
