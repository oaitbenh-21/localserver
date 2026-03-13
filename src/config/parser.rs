use std::fs;

#[derive(Debug, Clone)]
enum Token {
    Word(String),
    LBrace,
    RBrace,
    Semicolon,
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    for c in input.chars() {
        match c {
            '{' => {
                push_word(&mut tokens, &mut current);
                tokens.push(Token::LBrace);
            }
            '}' => {
                push_word(&mut tokens, &mut current);
                tokens.push(Token::RBrace);
            }
            ';' => {
                push_word(&mut tokens, &mut current);
                tokens.push(Token::Semicolon);
            }
            ' ' | '\n' | '\t' | '\r' => {
                push_word(&mut tokens, &mut current);
            }
            _ => current.push(c),
        }
    }

    push_word(&mut tokens, &mut current);

    tokens
}

fn push_word(tokens: &mut Vec<Token>, current: &mut String) {
    if !current.is_empty() {
        tokens.push(Token::Word(current.clone()));
        current.clear();
    }
}

#[derive(Debug)]
struct Config {
    servers: Vec<Server>,
}

#[derive(Debug)]
struct Server {
    host: String,
    port: u16,
    locations: Vec<Location>,
}

#[derive(Debug)]
struct Location {
    path: String,
    root: Option<String>,
    index: Option<String>,
    methods: Vec<String>,
    autoindex: bool,
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn parse_config(&mut self) -> Config {
        let mut servers = Vec::new();

        while let Some(token) = self.peek() {
            match token {
                Token::Word(word) if word == "server" => {
                    servers.push(self.parse_server());
                }
                _ => panic!("Unexpected token at config level"),
            }
        }

        Config { servers }
    }

    fn parse_server(&mut self) -> Server {
        self.advance(); // server

        self.expect_lbrace();

        let mut host = String::new();
        let mut port = 0;
        let mut locations = Vec::new();

        loop {
            match self.peek() {
                Some(Token::Word(word)) if word == "host" => {
                    self.advance();
                    host = self.parse_word();
                    self.expect_semicolon();
                }

                Some(Token::Word(word)) if word == "port" => {
                    self.advance();
                    port = self.parse_word().parse().unwrap();
                    self.expect_semicolon();
                }

                Some(Token::Word(word)) if word == "location" => {
                    locations.push(self.parse_location());
                }

                Some(Token::RBrace) => {
                    self.advance();
                    break;
                }

                _ => panic!("Invalid directive inside server"),
            }
        }

        Server {
            host,
            port,
            locations,
        }
    }

    fn parse_location(&mut self) -> Location {
        self.advance(); // location

        let path = self.parse_word();

        self.expect_lbrace();

        let mut root = None;
        let mut index = None;
        let mut methods = Vec::new();
        let mut autoindex = false;

        loop {
            match self.peek() {
                Some(Token::Word(word)) if word == "root" => {
                    self.advance();
                    root = Some(self.parse_word());
                    self.expect_semicolon();
                }

                Some(Token::Word(word)) if word == "index" => {
                    self.advance();
                    index = Some(self.parse_word());
                    self.expect_semicolon();
                }

                Some(Token::Word(word)) if word == "methods" => {
                    self.advance();
                    methods = self.parse_methods();
                    self.expect_semicolon();
                }

                Some(Token::Word(word)) if word == "autoindex" => {
                    self.advance();
                    autoindex = self.parse_word() == "on";
                    self.expect_semicolon();
                }

                Some(Token::RBrace) => {
                    self.advance();
                    break;
                }

                _ => panic!("Invalid directive in location"),
            }
        }

        Location {
            path,
            root,
            index,
            methods,
            autoindex,
        }
    }

    fn parse_methods(&mut self) -> Vec<String> {
        let mut methods = Vec::new();

        loop {
            match self.peek() {
                Some(Token::Word(word)) => {
                    methods.push(word.clone());
                    self.advance();
                }

                Some(Token::Semicolon) => break,

                _ => panic!("Invalid method"),
            }
        }

        methods
    }

    fn parse_word(&mut self) -> String {
        match self.peek() {
            Some(Token::Word(word)) => {
                let value = word.clone();
                self.advance();
                value
            }
            _ => panic!("Expected word"),
        }
    }

    fn expect_semicolon(&mut self) {
        match self.peek() {
            Some(Token::Semicolon) => self.advance(),
            _ => panic!("Expected ;"),
        }
    }

    fn expect_lbrace(&mut self) {
        match self.peek() {
            Some(Token::LBrace) => self.advance(),
            _ => panic!("Expected {{"),
        }
    }
}

fn main() {
    let config_text = fs::read_to_string("config.conf").expect("Cannot read config file");

    let tokens = tokenize(&config_text);

    let mut parser = Parser::new(tokens);

    let config = parser.parse_config();

    println!("{:#?}", config);
}
