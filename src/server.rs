// src/server.rs

use crate::epoll::{set_nonblocking, Epoll, MAX_EVENTS};
use crate::handler;
use crate::request::Request;
use crate::response::{Response, StatusCode};
use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::os::unix::io::AsRawFd;

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
        //  ___ Set up the listening socket ____________
        let listener = TcpListener::bind(&self.addr)?;
        println!("{:?}", listener);
        set_nonblocking(listener.as_raw_fd())?;
        println!("Server listening on http://{}", self.addr);

        //  Create epoll and register the listening socket __
        let epoll = Epoll::new()?;
        epoll.add(listener.as_raw_fd())?;
        println!("{:?}", epoll);

        //  Buffer to store per-connection incoming data __
        // Key = client fd, Value = bytes received so far
        let mut buffers: HashMap<i32, Vec<u8>> = HashMap::new();

        // The event loop ─────────────────────────────────────────────
        let mut events = vec![epoll_event { events: 0, u64: 0 }; MAX_EVENTS];

        loop {
            let ready = epoll.wait(&mut events)?;

            for i in 0..ready {
                let fd = events[i].u64 as i32;

                if fd == listener.as_raw_fd() {
                    // ── New connection arriving ────────────────────────────
                    self.accept_connections(&listener, &epoll, &mut buffers)?;
                } else {
                    // ── Existing client has data ───────────────────────────
                    self.handle_client(fd, &epoll, &mut buffers);
                }
            }
        }

        Ok(())
    }

    fn accept_connections() {}
    // pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
    //     let listener = TcpListener::bind(&self.addr)?;
    //     println!("Server listening on http://{}", self.addr);

    //     for stream in listener.incoming() {
    //         match stream {
    //             Ok(stream) => handle_connection(stream),
    //             Err(e) => eprintln!("Failed to accept connection: {}", e),
    //         }
    //     }

    //     Ok(())
    // }
}
