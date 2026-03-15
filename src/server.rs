// src/server.rs

use crate::epoll::{Epoll, MAX_EVENTS, set_nonblocking};
use crate::handler;
use crate::request::Request;
use crate::response::{Response, StatusCode};
use libc::epoll_event;
use std::collections::HashMap;
use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::os::unix::io::AsRawFd;
use std::time::Instant;

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
        let mut connect_times: HashMap<i32, Instant> = HashMap::new();

        // The event loop ─────────────────────────────────────────────
        let mut events = vec![epoll_event { events: 0, u64: 0 }; MAX_EVENTS];

        const TIMEOUT_SECS: u64 = 30;

        loop {
            let ready = epoll.wait(&mut events, 1000)?;
            // ── Check for timed out connections ──────────────────────────────
            let now = Instant::now();
            let mut timed_out: Vec<i32> = Vec::new();

            for (fd, connect_time) in connect_times.iter() {
                let elapsed = now.duration_since(*connect_time).as_secs();
                if elapsed > TIMEOUT_SECS {
                    timed_out.push(*fd);
                }
            }

            for fd in timed_out {
                eprintln!("Connection {} timed out", fd);
                let _ = epoll.remove(fd);
                buffers.remove(&fd);
                connect_times.remove(&fd);
                unsafe { libc::close(fd) };
            }
            // ── Handle ready events ───────────────────────────────────────
            for i in 0..ready {
                let fd = events[i].u64 as i32;

                if fd == listener.as_raw_fd() {
                    self.accept_connections(&listener, &epoll, &mut buffers, &mut connect_times)?;
                } else {
                    self.handle_client(fd, &epoll, &mut buffers);
                    connect_times.remove(&fd); // ← remove when connection is handled
                }
            }
        }

        Ok(())
    }

    fn accept_connections(
        &self,
        listener: &TcpListener,
        epoll: &Epoll,
        buffers: &mut HashMap<i32, Vec<u8>>,
        connect_times: &mut HashMap<i32, Instant>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("accepting connection");
        // With edge-triggered we must accept in a loop until WouldBlock
        loop {
            match listener.accept() {
                Ok((stream, addr)) => {
                    // stream --> the connection socket
                    println!("New connection: {}", addr);
                    let fd = stream.as_raw_fd();

                    // Set non-blocking BEFORE adding to epoll
                    set_nonblocking(fd)?;
                    epoll.add(fd)?;

                    // Initialize an empty buffer for this client
                    buffers.insert(fd, Vec::new());
                    // time of registration
                    connect_times.insert(fd, Instant::now());
                    // Prevent Rust from closing the socket when
                    // stream drops at end of this block
                    std::mem::forget(stream);
                }

                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No more incoming connections right now — stop looping
                    break;
                }
                Err(e) => {
                    eprintln!("Accept error: {}", e);
                    break;
                }
            }
        }
        Ok(())
    }
    fn handle_client(&self, fd: i32, epoll: &Epoll, buffers: &mut HashMap<i32, Vec<u8>>) {
        let mut buf = [0u8; 4096];
        let mut stream = unsafe {
            use std::os::unix::io::FromRawFd;
            std::net::TcpStream::from_raw_fd(fd)
        };

        // ── Read loop — drain the entire buffer ───────────────────────────
        loop {
            match stream.read(&mut buf) {
                Ok(0) => {
                    // Client disconnected
                    println!("Client {} disconnected", fd);
                    let _ = epoll.remove(fd);
                    buffers.remove(&fd);
                    return;
                }
                Ok(n) => {
                    // Append new bytes to this client's buffer
                    if let Some(buffer) = buffers.get_mut(&fd) {
                        buffer.extend_from_slice(&buf[..n]);
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // Buffer fully drained — now process what we have
                    break;
                }
                Err(e) => {
                    eprintln!("Read error on fd {}: {}", fd, e);
                    let _ = epoll.remove(fd);
                    buffers.remove(&fd);
                    return;
                }
            }
        }
        // ── Process the request ───────────────────────────────────────────
        if let Some(data) = buffers.get(&fd) {
            match Request::parse(data) {
                Some(req) => {
                    println!("Method: {:?}, Path: {}", req.method, req.path);
                    handler::handle(req, &mut stream);
                }
                None => {
                    Response::error(StatusCode::BadRequest).send(&mut stream);
                }
            }
        }
        // ── Clean up after responding ─────────────────────────────────────
        let _ = epoll.remove(fd);
        buffers.remove(&fd);

        // Prevent double-close — we'll manage this fd manually
        std::mem::forget(stream);
    }
}
