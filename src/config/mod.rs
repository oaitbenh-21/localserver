pub mod parser;
pub mod tokenizer;

use std::collections::HashMap;

#[derive(Debug)]
pub struct Config {
    pub servers: Vec<ServerConfig>,
}

#[derive(Debug)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub client_max_body_size: usize,       // in bytes
    pub error_pages: HashMap<u16, String>, // e.g. 404 → "./error_pages/404.html"
    pub locations: Vec<Location>,
}

#[derive(Debug)]
pub struct Location {
    pub path: String,
    pub root: String,
    pub index: Option<String>,
    pub methods: Vec<Method>,
    pub autoindex: bool,
    pub redirect: Option<String>,
    pub cgi: Option<CGI>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Method {
    Get,
    Post,
    Delete,
}

#[derive(Debug)]
pub struct CGI {
    pub extension: String,
    pub interpreter: String,
}
