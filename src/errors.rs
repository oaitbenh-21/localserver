// ── Error type ────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub line: usize, // which token position the error occurred at
}

impl ParseError {
    pub fn new(message: impl Into<String>, pos: usize) -> ParseError {
        ParseError {
            message: message.into(),
            line: pos,
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Config parse error at token {}: {}",
            self.line, self.message
        )
    }
}

impl std::error::Error for ParseError {} // have a debug nd desplay to join the club or errors. 

pub type ParseResult<T> = Result<T, ParseError>;
