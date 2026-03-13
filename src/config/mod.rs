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

struct Location {
    path: String,
    root: String,
    index: Option<String>,
    methods: Vec<Method>,
    autoindex: bool,
    redirect: Option<String>,
    cgi: Option<CGI>,
}

enum Method {
    GET,
    POST,
    DELETE,
}

struct CGI {
    extension: String,
    interpreter: String,
}
