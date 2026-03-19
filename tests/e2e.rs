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
