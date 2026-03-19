// src/handler.rs

use crate::request::{Method, Request};
use crate::response::{Response, StatusCode};
use std::fs;
use std::net::TcpStream;

fn get_content_type(path: &str) -> &str {
    if path.ends_with(".html") {
        "text/html"
    } else if path.ends_with(".css") {
        "text/css"
    } else if path.ends_with(".js") {
        "application/javascript"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".jpg") {
        "image/jpeg"
    } else if path.ends_with(".json") {
        "application/json"
    } else if path.ends_with(".txt") {
        "text/plain"
    } else {
        "application/octet-stream"
    }
}

fn serve_file(path: &str, root: &str) -> Response {
    let normalized = if path.ends_with('/') {
        format!("{}index.html", path)
    } else {
        path.to_string()
    };

    let file_path = format!("{}{}", root, normalized);

    match fs::read(&file_path) {
        Ok(contents) => {
            let content_type = get_content_type(&normalized);
            Response::new(StatusCode::Ok, content_type, contents)
        }
        Err(_) => Response::error(StatusCode::NotFound),
    }
}

fn handle_post(req: &Request, root: &str) -> Response {
    // Reject empty bodies
    if req.body.is_empty() {
        return Response::error(StatusCode::BadRequest);
    }

    // Build a safe file path from the URL path
    // POST /upload/photo.png → saves to www/upload/photo.png
    let file_path = format!("{}{}", root, req.path);

    // Make sure the directory exists
    if let Some(parent) = std::path::Path::new(&file_path).parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            eprintln!("Failed to create directory: {}", e);
            return Response::error(StatusCode::InternalServerError);
        }
    }

    // Write the body to disk
    match fs::write(&file_path, &req.body) {
        Ok(_) => {
            let body = format!(
                "<html><body><h1>Uploaded successfully to {}</h1></body></html>",
                req.path
            );
            Response::new(StatusCode::Ok, "text/html", body.into_bytes())
        }
        Err(e) => {
            eprintln!("Failed to write file: {}", e);
            Response::error(StatusCode::InternalServerError)
        }
    }
}

fn handle_delete(req: &Request, root: &str) -> Response {
    let file_path = format!("{}{}", root, req.path);

    match fs::remove_file(&file_path) {
        Ok(_) => {
            let body = format!("<html><body><h1>Deleted {}</h1></body></html>", req.path);
            Response::new(StatusCode::Ok, "text/html", body.into_bytes())
        }
        Err(_) => Response::error(StatusCode::NotFound),
    }
}

pub fn handle(req: Request, stream: &mut TcpStream) {
    handle_with_root(req, stream, "www");
}

pub fn handle_with_root(req: Request, stream: &mut TcpStream, root: &str) {
    let response = match req.method {
        Method::Get => serve_file(&req.path, root),
        Method::Post => handle_post(&req, root),
        Method::Delete => handle_delete(&req, root),
        Method::Unknown(_) => Response::error(StatusCode::MethodNotAllowed),
    };
    response.send(stream);
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::request::{Method, Request};
    use std::fs;
    use std::io::Read;
    use std::net::{TcpListener, TcpStream};

    // ── Helpers ───────────────────────────────────────────────────────────

    // Creates a temp directory unique to each test
    fn temp_dir(name: &str) -> std::path::PathBuf {
        let path = std::path::PathBuf::from(format!("/tmp/localserver_test_{}", name));
        let _ = fs::remove_dir_all(&path); // clean up any previous run
        fs::create_dir_all(&path).unwrap();
        path
    }

    // Builds a minimal GET request struct
    fn get(path: &str) -> Request {
        Request {
            method: Method::Get,
            path: path.to_string(),
            version: "HTTP/1.1".to_string(),
            headers: std::collections::HashMap::new(),
            body: Vec::new(),
        }
    }

    // Builds a minimal POST request struct with a body
    fn post(path: &str, body: &[u8]) -> Request {
        Request {
            method: Method::Post,
            path: path.to_string(),
            version: "HTTP/1.1".to_string(),
            headers: std::collections::HashMap::new(),
            body: body.to_vec(),
        }
    }

    // Builds a minimal DELETE request struct
    fn delete(path: &str) -> Request {
        Request {
            method: Method::Delete,
            path: path.to_string(),
            version: "HTTP/1.1".to_string(),
            headers: std::collections::HashMap::new(),
            body: Vec::new(),
        }
    }

    // Runs handler::handle and captures raw bytes sent over wire
    fn capture(req: Request, root: &str) -> Vec<u8> {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let handle = std::thread::spawn(move || {
            let mut client = TcpStream::connect(addr).unwrap();
            let mut buf = Vec::new();
            client.read_to_end(&mut buf).unwrap();
            buf
        });

        let (mut stream, _) = listener.accept().unwrap();
        handle_with_root(req, &mut stream, root); // send request, the response is captured at the thread. 
        drop(stream); // so now we can just close the connection  to inform the thread we are done.  this signals EOF 

        handle.join().unwrap()
    }
    // Extracts the status line from raw response bytes
    fn status_line(bytes: &[u8]) -> String {
        let raw = String::from_utf8_lossy(bytes);
        raw.lines().next().unwrap_or("").to_string()
    }
    // Extracts the body from raw response bytes
    fn body(bytes: &[u8]) -> Vec<u8> {
        let separator = b"\r\n\r\n";
        let pos = bytes.windows(4).position(|w| w == separator).unwrap();
        bytes[pos + 4..].to_vec()
    }

    // Extracts a specific header value from raw response bytes
    fn header<'a>(bytes: &'a [u8], name: &str) -> Option<String> {
        let raw = String::from_utf8_lossy(bytes);
        for line in raw.lines() {
            if line.to_lowercase().starts_with(&name.to_lowercase()) {
                return Some(line.splitn(2, ':').nth(1)?.trim().to_string());
            }
        }
        None
    }

    // ── GET tests ─────────────────────────────────────────────────────────

    #[test]
    fn test_get_existing_file() {
        let root = temp_dir("get_existing");
        fs::write(root.join("index.html"), b"<h1>hello</h1>").unwrap();

        let bytes = capture(get("/index.html"), root.to_str().unwrap());
        assert!(status_line(&bytes).contains("200 OK"));
        assert_eq!(body(&bytes), b"<h1>hello</h1>");
    }
    #[test]
    fn test_get_root_serves_index_html() {
        let root = temp_dir("get_root");
        fs::write(root.join("index.html"), b"<h1>home</h1>").unwrap();

        let bytes = capture(get("/"), root.to_str().unwrap());
        assert!(status_line(&bytes).contains("200 OK"));
        assert_eq!(body(&bytes), b"<h1>home</h1>");
    }

    #[test]
    fn test_get_missing_file_returns_404() {
        let root = temp_dir("get_missing");

        let bytes = capture(get("/missing.html"), root.to_str().unwrap());
        assert!(status_line(&bytes).contains("404 Not Found"));
    }

    #[test]
    fn test_get_correct_content_type_css() {
        let root = temp_dir("get_ct_css");
        fs::write(root.join("style.css"), b"body{}").unwrap();

        let bytes = capture(get("/style.css"), root.to_str().unwrap());
        assert_eq!(header(&bytes, "content-type").as_deref(), Some("text/css"));
    }

    #[test]
    fn test_get_correct_content_type_js() {
        let root = temp_dir("get_ct_js");
        fs::write(root.join("app.js"), b"console.log('hi')").unwrap();

        let bytes = capture(get("/app.js"), root.to_str().unwrap());
        assert_eq!(
            header(&bytes, "content-type").as_deref(),
            Some("application/javascript")
        );
    }
    #[test]
    fn test_get_binary_file_survives() {
        let root = temp_dir("get_binary");
        let data = vec![0xFF, 0xD8, 0xFF, 0xE0]; // JPEG magic bytes
        fs::write(root.join("img.jpg"), &data).unwrap();

        let bytes = capture(get("/img.jpg"), root.to_str().unwrap());
        assert!(status_line(&bytes).contains("200 OK"));
        assert_eq!(body(&bytes), data);
    }
    #[test]
    fn test_content_length_matches_file_size() {
        let root = temp_dir("get_content_length");
        let content = b"exactly nineteen!";
        fs::write(root.join("file.txt"), content).unwrap();

        let bytes = capture(get("/file.txt"), root.to_str().unwrap());
        let len = header(&bytes, "content-length").unwrap();
        assert_eq!(len, content.len().to_string());
    }
}
