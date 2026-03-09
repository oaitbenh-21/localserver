// src/main.rs

use std::io::Read;
use std::net::{TcpListener, TcpStream};

mod handler;
mod request;
mod response;

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    if let Err(e) = stream.read(&mut buffer) {
        eprintln!("Failed to read from connection: {}", e);
        return;
    }

    match request::Request::parse(&buffer) {
        Some(req) => {
            println!("Method: {:?}, Path: {}", req.method, req.path);
            handler::handle(req, &mut stream);
        }
        None => {
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
