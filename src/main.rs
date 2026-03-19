// src/main.rs




fn main() -> Result<(), Box<dyn std::error::Error>> {
    let srv = localserver::server::Server::new("127.0.0.1:8080");
    srv.run()
}
