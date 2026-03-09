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
    } else {
        "application/octet-stream"
    }
}
fn serve_file(path: &str) -> Response {
    // If path ends with "/" append "index.html", otherwise use as-is
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

pub fn handle(req: Request, stream: &mut TcpStream) {
    println!("Headers: {:?}", req.headers);
    let response = match req.method {
        Method::Get => serve_file(&req.path),
        Method::Post | Method::Delete => Response::error(StatusCode::MethodNotAllowed),
        Method::Unknown(_) => Response::error(StatusCode::MethodNotAllowed),
    };

    response.send(stream);
}
