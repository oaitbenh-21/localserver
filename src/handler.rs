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

fn serve_file(path: &str) -> Response {
    let normalized = if path.ends_with('/') {
        format!("{}index.html", path)
    } else {
        path.to_string()
    };

    let file_path = format!("www{}", normalized);

    match fs::read(&file_path) {
        Ok(contents) => {
            let content_type = get_content_type(&normalized);
            Response::new(StatusCode::Ok, content_type, contents)
        }
        Err(_) => Response::error(StatusCode::NotFound),
    }
}

fn handle_post(req: &Request) -> Response {
    // Reject empty bodies
    if req.body.is_empty() {
        return Response::error(StatusCode::BadRequest);
    }

    // Build a safe file path from the URL path
    // POST /upload/photo.png → saves to www/upload/photo.png
    let file_path = format!("www{}", req.path);

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

fn handle_delete(req: &Request) -> Response {
    let file_path = format!("www{}", req.path);

    match fs::remove_file(&file_path) {
        Ok(_) => {
            let body = format!("<html><body><h1>Deleted {}</h1></body></html>", req.path);
            Response::new(StatusCode::Ok, "text/html", body.into_bytes())
        }
        Err(_) => Response::error(StatusCode::NotFound),
    }
}

pub fn handle(req: Request, stream: &mut TcpStream) {
    let response = match req.method {
        Method::Get => serve_file(&req.path),
        Method::Post => handle_post(&req),
        Method::Delete => handle_delete(&req),
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
}
