mod request;
mod response;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use std::fs;
use std::path::Path;

fn serve_file(stream: &mut TcpStream, path: &str) {
    // Strip the leading "/" and prefix with our www folder
    let file_path = format!("www{}", path);

    match fs::read(&file_path) {
        Ok(contents) => {
            let response_header = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/html\r\n\r\n",
                contents.len()
            );
            if let Err(e) = stream.write_all(response_header.as_bytes()) {
                eprintln!("Failed to write header: {}", e);
                return;
            }
            if let Err(e) = stream.write_all(&contents) {
                eprintln!("Failed to write body: {}", e);
            }
        }
        Err(_) => {
            let body = "<html><body><h1>404 - Not Found</h1></body></html>";
            let response = format!(
                "HTTP/1.1 404 Not Found\r\nContent-Length: {}\r\nContent-Type: text/html\r\n\r\n{}",
                body.len(),
                body
            );
            if let Err(e) = stream.write_all(response.as_bytes()) {
                eprintln!("Failed to write 404: {}", e);
            }
        }
    }
}

fn parse_request_line(buffer: &[u8]) -> Option<(String, String, String)> {
    let request = String::from_utf8_lossy(buffer);
    let first_line = request.lines().next()?;
    let mut parts = first_line.split_whitespace();

    let method = parts.next()?.to_string();
    let path = parts.next()?.to_string();
    let version = parts.next()?.to_string();

    Some((method, path, version))
}
fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    if let Err(e) = stream.read(&mut buffer) {
        eprintln!("Failed to read from connection: {}", e);
        return;
    }

    match request::Request::parse(&buffer) {
        Some(req) => {
            println!("Method: {:?}, Path: {}", req.method, req.path);
            serve_file(&mut stream, &req.path);
        }

        None => {
            eprintln!("Could not parse request");
            response::Response::error(response::StatusCode::BadRequest).send(&mut stream);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;

    println!("Server listening on http://localhost:8080");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_connection(stream),
            Err(e) => eprintln!("Failed to accept connection: {}", e),
        }
    }

    Ok(())
}
