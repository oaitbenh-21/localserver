use std::str::FromStr;

use super::tokenizer::Token;
use super::{CGI, Config, Location, Method, ServerConfig};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<Token> {
        let tok = self.tokens.get(self.pos).cloned();
        self.pos += 1;
        tok
    }

    fn expect_word(&mut self) -> String {
        match self.next() {
            Some(Token::Word(w)) => w,
            _ => panic!("Expected word"),
        }
    }

    fn expect(&mut self, expected: Token) {
        let tok = self.next();
        if tok != Some(expected) {
            panic!("Unexpected token: {:?}", tok);
        }
    }

    pub fn parse_config(&mut self) -> Config {
        let mut servers = Vec::new();

        while self.peek().is_some() {
            let word = self.expect_word();
            if word == "server" {
                servers.push(self.parse_server());
            } else {
                panic!("Unexpected directive: {}", word);
            }
        }

        Config { servers }
    }

    fn parse_server(&mut self) -> ServerConfig {
        self.expect(Token::LBrace);

        let mut host = String::new();
        let mut port = 0;
        let mut locations = Vec::new();

        while let Some(tok) = self.peek() {
            match tok {
                Token::RBrace => {
                    self.next();
                    break;
                }
                Token::Word(_) => {
                    let directive = self.expect_word();
                    match directive.as_str() {
                        "host" => {
                            host = self.expect_word();
                            self.expect(Token::Semicolon);
                        }
                        "port" => {
                            port = u16::from_str(&self.expect_word()).unwrap();
                            self.expect(Token::Semicolon);
                        }
                        "location" => {
                            locations.push(self.parse_location());
                        }
                        _ => {
                            // skip unknown
                            while self.next() != Some(Token::Semicolon) {}
                        }
                    }
                }
                _ => panic!("Unexpected token in server"),
            }
        }

        ServerConfig {
            host,
            port,
            locations,
            todo!()
        }
    }

    fn parse_location(&mut self) -> Location {
        let path = self.expect_word();
        self.expect(Token::LBrace);

        let mut root = String::new();
        let mut index = None;
        let mut methods = Vec::new();
        let mut autoindex = false;
        let mut redirect = None;
        let mut cgi = None;

        while let Some(tok) = self.peek() {
            match tok {
                Token::RBrace => {
                    self.next();
                    break;
                }
                Token::Word(_) => {
                    let directive = self.expect_word();
                    match directive.as_str() {
                        "root" => {
                            root = self.expect_word();
                            self.expect(Token::Semicolon);
                        }
                        "index" => {
                            index = Some(self.expect_word());
                            self.expect(Token::Semicolon);
                        }
                        "methods" => {
                            while let Some(Token::Word(_)) = self.peek() {
                                let m = self.expect_word();
                                methods.push(match m.as_str() {
                                    "GET" => Method::GET,
                                    "POST" => Method::POST,
                                    "DELETE" => Method::DELETE,
                                    _ => panic!("Unknown method"),
                                });
                            }
                            self.expect(Token::Semicolon);
                        }
                        "autoindex" => {
                            autoindex = self.expect_word() == "on";
                            self.expect(Token::Semicolon);
                        }
                        "redirect" => {
                            redirect = Some(self.expect_word());
                            self.expect(Token::Semicolon);
                        }
                        "cgi" => {
                            let ext = self.expect_word();
                            self.expect(Token::Semicolon);

                            cgi = Some(CGI {
                                extension: ext,
                                interpreter: "python3".into(),
                            });
                        }
                        _ => while self.next() != Some(Token::Semicolon) {},
                    }
                }
                _ => panic!("Unexpected token in location"),
            }
        }

        Location {
            path,
            root,
            index,
            methods,
            autoindex,
            redirect,
            cgi,
        }
    }
}
