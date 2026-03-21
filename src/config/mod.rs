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

impl Config {
    pub fn from_file(path: &str) -> Result<Config, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Cannot read config file '{}': {}", path, e))?;

        let tokens = tokenizer::tokenize(&content);
        let mut parser = parser::Parser::new(tokens);
        parser.parse_config()
    }
}

impl ServerConfig {
    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

// Converts "10MB", "1kb", "500" etc. into bytes
pub fn parse_body_size(s: &str) -> Result<usize, String> {
    let s = s.to_lowercase();

    if let Some(n) = s.strip_suffix("mb") {
        n.trim()
            .parse::<usize>()
            .map(|n| n * 1024 * 1024)
            .map_err(|_| format!("Invalid size: {}", s))
    } else if let Some(n) = s.strip_suffix("kb") {
        n.trim()
            .parse::<usize>()
            .map(|n| n * 1024)
            .map_err(|_| format!("Invalid size: {}", s))
    } else {
        s.trim()
            .parse::<usize>()
            .map_err(|_| format!("Invalid size: {}", s))
    }
}
