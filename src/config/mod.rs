pub mod tokenizer;
pub mod parser;

#[derive(Debug)]
pub struct Config {
    pub servers: Vec<Server>,
}

#[derive(Debug)]
pub struct Server {
    pub host: String,
    pub port: u16,
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

#[derive(Debug)]
pub enum Method {
    GET,
    POST,
    DELETE,
}

#[derive(Debug)]
pub struct CGI {
    pub extension: String,
    pub interpreter: String,
}