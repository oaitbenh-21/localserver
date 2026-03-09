use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn parse_request_line(buffer: &[u8]) -> Option<(String, String, String)> {
    // Convert raw bytes to a string
    let request = String::from_utf8_lossy(buffer);

    // The first line ends at the first \r\n
    let first_line = request.lines().next()?;

    // Split by space — should give us ["GET", "/path", "HTTP/1.1"]
    let mut parts = first_line.split_whitespace();

    let method = parts.next()?.to_string();
    let path = parts.next()?.to_string();
    let version = parts.next()?.to_string();

    Some((method, path, version))
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    match parse_request_line(&buffer) {
        Some((method, path, version)) => {
            println!("Method: {}, Path: {}, Version: {}", method, path, version);

            let response = "HTTP/1.1 200 OK\r\nContent-Length: 13\r\n\r\nHello, World!";
            stream.write_all(response.as_bytes()).unwrap();
        }
        None => {
            println!("Could not parse request");
            let response = "HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n";
            stream.write_all(response.as_bytes()).unwrap();
        }
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8081").unwrap();
    println!("Server listening on http://localhost:8081");

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream);
    }
}
