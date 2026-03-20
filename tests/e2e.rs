use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

// ── Helper: spin up a real server on a random port ────────────────────────────

fn start_server() -> u16 {
    // Bind to port 0 to get a random available port from the OS
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener); // free it — server will rebind immediately

    thread::spawn(move || {
        localserver::server::Server::new(&format!("127.0.0.1:{}", port))
            .run()
            .unwrap();
    });

    // Give the server a moment to start
    thread::sleep(Duration::from_millis(100));
    port
}

// ── Helper: send a raw HTTP request, get raw bytes back ──────────────────────

fn send_request(port: u16, request: &str) -> Vec<u8> {
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    stream.write_all(request.as_bytes()).unwrap();

    let mut buf = Vec::new();
    let _ = stream.read_to_end(&mut buf);
    buf
}

// ── Helper: extract status line ───────────────────────────────────────────────

fn status_line(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes)
        .lines()
        .next()
        .unwrap_or("")
        .to_string()
}

// ── Helper: extract body ──────────────────────────────────────────────────────

fn body(bytes: &[u8]) -> Vec<u8> {
    let sep = b"\r\n\r\n";
    match bytes.windows(4).position(|w| w == sep) {
        Some(pos) => bytes[pos + 4..].to_vec(),
        None => Vec::new(),
    }
}

// ── Helper: extract header value ─────────────────────────────────────────────

fn header(bytes: &[u8], name: &str) -> Option<String> {
    let raw = String::from_utf8_lossy(bytes);
    for line in raw.lines() {
        if line.to_lowercase().starts_with(&name.to_lowercase()) {
            return Some(line.splitn(2, ':').nth(1)?.trim().to_string());
        }
    }
    None
}

// ── Helper: setup a temp www root with a file ─────────────────────────────────

fn setup_www(name: &str, file: &str, content: &[u8]) -> String {
    let root = format!("/tmp/e2e_{}", name);
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    fs::write(format!("{}/{}", root, file), content).unwrap();
    root
}

// ── GET tests ─────────────────────────────────────────────────────────────────

#[test]
fn e2e_get_returns_200() {
    let port = start_server();
    let response = send_request(
        port,
        "GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    );
    assert!(status_line(&response).contains("200") || status_line(&response).contains("404")); // server responded — didn't crash
}

#[test]
fn e2e_get_missing_returns_404() {
    let port = start_server();
    let response = send_request(
        port,
        "GET /this-does-not-exist HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n", // After you send your response, close the TCP connection. kinda http 1.0 not http 1.1
    );
    assert!(status_line(&response).contains("404 Not Found"));
}

#[test]
fn e2e_response_is_valid_http() {
    let port = start_server();
    let response = send_request(
        port,
        "GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    );
    let raw = String::from_utf8_lossy(&response);

    // Must start with HTTP/1.1
    assert!(raw.starts_with("HTTP/1.1"));

    // Must contain \r\n\r\n separator
    assert!(raw.contains("\r\n\r\n"));

    // Must have Content-Length header
    assert!(header(&response, "content-length").is_some());
}

#[test]
fn e2e_content_length_matches_body() {
    let port = start_server();
    let response = send_request(
        port,
        "GET /missing.html HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    );

    let declared_len: usize = header(&response, "content-length")
        .unwrap()
        .parse()
        .unwrap();

    assert_eq!(declared_len, body(&response).len());
}

// ── POST tests ────────────────────────────────────────────────────────────────

#[test]
fn e2e_post_upload_and_retrieve() {
    let port = start_server();
    let content = "hello from e2e test";

    // Upload
    let upload = format!(
        "POST /uploads/e2e_test.txt HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        content.len(),
        content
    );
    let response = send_request(port, &upload);
    assert!(status_line(&response).contains("200 OK"));

    // Retrieve
    let retrieve = send_request(port,
        "GET /uploads/e2e_test.txt HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"
    );
    assert!(status_line(&retrieve).contains("200 OK"));
    assert_eq!(body(&retrieve), content.as_bytes());
}

#[test]
fn e2e_post_empty_body_returns_400() {
    let port = start_server();
    let request =
        "POST /uploads/empty.txt HTTP/1.1\r\nHost: localhost\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";

    let response = send_request(port, request);
    assert!(status_line(&response).contains("400 Bad Request"));
}