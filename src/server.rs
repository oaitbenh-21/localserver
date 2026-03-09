// src/server.rs

use crate::handler;
use crate::request::Request;
use crate::response::{Response, StatusCode};
use std::io::Read;
use std::net::{TcpListener, TcpStream};

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    if let Err(e) = stream.read(&mut buffer) {
        eprintln!("Failed to read from connection: {}", e);
        return;
    }

    match Request::parse(&buffer) {
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
