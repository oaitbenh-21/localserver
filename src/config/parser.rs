use super::tokenizer::Token;
use super::{CGI, Location, Method, ServerConfig, parse_body_size};
use crate::errors::{ParseError, ParseResult};
use std::collections::HashMap;
use std::str::FromStr;

// ── Parser ────────────────────────────────────────────────────────────────────

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens, pos: 0 }
    }

    // ── Low level token operations ────────────────────────────────────────

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<Token> {
        let tok = self.tokens.get(self.pos).cloned();
        self.pos += 1;
        tok
    }

    // Consume the next token and return it if it's a Word, else error
    fn expect_word(&mut self) -> ParseResult<String> {
        match self.next() {
            Some(Token::Word(w)) => Ok(w),
            Some(other) => Err(ParseError::new(
                format!("Expected a value, got {:?}", other),
                self.pos,
            )),
            None => Err(ParseError::new("Unexpected end of config", self.pos)),
        }
    }

    // Consume the next token and verify it matches what we expect
    fn expect(&mut self, expected: Token) -> ParseResult<()> {
        match self.next() {
            Some(tok) if tok == expected => Ok(()),
            Some(other) => Err(ParseError::new(
                format!("Expected {:?}, got {:?}", expected, other),
                self.pos,
            )),
            None => Err(ParseError::new(
                format!("Expected {:?} but reached end of config", expected),
                self.pos,
            )),
        }
    }

    // ── Top level ─────────────────────────────────────────────────────────

    pub fn parse_config(&mut self) -> ParseResult<super::Config> {
        let mut servers = Vec::new();

        while self.peek().is_some() {
            let word = self.expect_word()?;
            match word.as_str() {
                "server" => servers.push(self.parse_server()?),
                other => {
                    return Err(ParseError::new(
                        format!("Unknown top-level directive '{}'", other),
                        self.pos,
                    ));
                }
            }
        }

        if servers.is_empty() {
            return Err(ParseError::new("Config must define at least one server", 0));
        }

        Ok(super::Config { servers })
    }

    // ── Server block ──────────────────────────────────────────────────────

    fn parse_server(&mut self) -> ParseResult<ServerConfig> {
        self.expect(Token::LBrace)?;

        let mut host = None;
        let mut port = None;
        let mut client_max_body_size = 1024 * 1024; // default 1MB
        let mut error_pages = HashMap::new();
        let mut locations = Vec::new();

        while let Some(tok) = self.peek() {
            match tok {
                Token::RBrace => {
                    self.next();
                    break;
                }
                Token::Word(_) => {
                    let directive = self.expect_word()?;
                    match directive.as_str() {
                        "host" => {
                            host = Some(self.expect_word()?);
                            self.expect(Token::Semicolon)?;
                        }
                        "port" => {
                            let raw = self.expect_word()?;
                            port = Some(u16::from_str(&raw).map_err(|_| {
                                ParseError::new(format!("Invalid port number '{}'", raw), self.pos)
                            })?);
                            self.expect(Token::Semicolon)?;
                        }
                        "client_max_body_size" => {
                            let raw = self.expect_word()?;
                            client_max_body_size =
                                parse_body_size(&raw).map_err(|e| ParseError::new(e, self.pos))?;
                            self.expect(Token::Semicolon)?;
                        }
                        "error_page" => {
                            let code_str = self.expect_word()?;
                            let code = code_str.parse::<u16>().map_err(|_| {
                                ParseError::new(
                                    format!("Invalid error code '{}'", code_str),
                                    self.pos,
                                )
                            })?;
                            let path = self.expect_word()?;
                            error_pages.insert(code, path);
                            self.expect(Token::Semicolon)?;
                        }
                        "location" => {
                            locations.push(self.parse_location()?);
                        }
                        other => {
                            return Err(ParseError::new(
                                format!("Unknown server directive '{}'", other),
                                self.pos,
                            ));
                        }
                    }
                }
                other => {
                    return Err(ParseError::new(
                        format!("Unexpected token {:?} in server block", other),
                        self.pos,
                    ));
                }
            }
        }
        // Validate required fields
        let host = host.ok_or_else(|| ParseError::new("Server block missing 'host'", self.pos))?;
        let port = port.ok_or_else(|| ParseError::new("Server block missing 'port'", self.pos))?;

        Ok(ServerConfig {
            host,
            port,
            client_max_body_size,
            error_pages,
            locations,
        })
    }
    // ── Location block ────────────────────────────────────────────────────

    fn parse_location(&mut self) -> ParseResult<Location> {
        let path = self.expect_word()?;
        self.expect(Token::LBrace)?;

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
                    let directive = self.expect_word()?;
                    match directive.as_str() {
                        "root" => {
                            root = self.expect_word()?;
                            self.expect(Token::Semicolon)?;
                        }
                        "index" => {
                            index = Some(self.expect_word()?);
                            self.expect(Token::Semicolon)?;
                        }
                        "methods" => {
                            // Read all method words until we hit a semicolon
                            while let Some(Token::Word(_)) = self.peek() {
                                let m = self.expect_word()?;
                                let method = match m.as_str() {
                                    "GET" => Method::Get,
                                    "POST" => Method::Post,
                                    "DELETE" => Method::Delete,
                                    other => {
                                        return Err(ParseError::new(
                                            format!("Unknown method '{}'", other),
                                            self.pos,
                                        ));
                                    }
                                };
                                methods.push(method);
                            }
                            self.expect(Token::Semicolon)?;
                        }
                        "autoindex" => {
                            let val = self.expect_word()?;
                            autoindex = match val.as_str() {
                                "on" => true,
                                "off" => false,
                                other => {
                                    return Err(ParseError::new(
                                        format!("autoindex must be 'on' or 'off', got '{}'", other),
                                        self.pos,
                                    ));
                                }
                            };
                            self.expect(Token::Semicolon)?;
                        }
                        "redirect" => {
                            redirect = Some(self.expect_word()?);
                            self.expect(Token::Semicolon)?;
                        }
                        "cgi" => {
                            let ext = self.expect_word()?;
                            let interpreter = self.expect_word()?;
                            cgi = Some(CGI {
                                extension: ext,
                                interpreter,
                            });
                            self.expect(Token::Semicolon)?;
                        }
                        other => {
                            return Err(ParseError::new(
                                format!("Unknown location directive '{}'", other),
                                self.pos,
                            ));
                        }
                    }
                }
                other => {
                    return Err(ParseError::new(
                        format!("Unexpected token {:?} in location block", other),
                        self.pos,
                    ));
                }
            }
        }

        // Validate — a location must have at least a path
        // root can be empty for redirect-only locations
        if path.is_empty() {
            return Err(ParseError::new("Location path cannot be empty", self.pos));
        }

        Ok(Location {
            path,
            root,
            index,
            methods,
            autoindex,
            redirect,
            cgi,
        })
    }
}
