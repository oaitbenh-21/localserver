// src/server.rs

use crate::handler;
use crate::request::Request;
use crate::response::{Response, StatusCode};
use std::io::Read;
use std::net::{TcpListener, TcpStream};

fn handle_connection(mut stream: TcpStream) {
    // Step 1 — read headers first (until \r\n\r\n)
    let mut header_buf = Vec::new();
    let mut byte = [0u8; 1];

    loop {
        match stream.read(&mut byte) {
            Ok(0) => return, // connection closed
            Ok(_) => {
                header_buf.push(byte[0]);
                if header_buf.ends_with(b"\r\n\r\n") {
                    break;
                }
            }
            Err(e) => {
                eprintln!("Failed to read headers: {}", e);
                return;
            }
        }
    }

    // Step 2 — parse what we have so far to get Content-Length
    let partial_req = Request::parse(&header_buf);

    let content_length = match &partial_req {
        Some(req) => req.content_length(),
        None => {
            Response::error(StatusCode::BadRequest).send(&mut stream);
            return;
        }
    };

    // Step 3 — read the body if there is one
    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        if let Err(e) = stream.read_exact(&mut body) {
            eprintln!("Failed to read body: {}", e);
            return;
        }
    }

    // Step 4 — combine headers + body and parse the full request
    let mut full_request = header_buf;
    full_request.extend_from_slice(&body);

    match Request::parse(&full_request) {
        Some(req) => {
            println!("Method: {:?}, Path: {}", req.method, req.path);
            handler::handle(req, &mut stream);
        }
        None => {
            Response::error(StatusCode::BadRequest).send(&mut stream);
        }
    }
}

pub struct Server {
    addr: String,
}

impl Server {
    pub fn new(addr: &str) -> Server {
        Server {
            addr: addr.to_string(),
        }
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(&self.addr)?;
        println!("Server listening on http://{}", self.addr);

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => handle_connection(stream),
                Err(e) => eprintln!("Failed to accept connection: {}", e),
            }
        }

        Ok(())
    }
}
