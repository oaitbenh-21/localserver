// src/main.rs

mod handler;
mod request;
mod response;
mod server;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let srv = server::Server::new("127.0.0.1:8080");
    srv.run()
}
