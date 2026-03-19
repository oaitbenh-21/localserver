#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Word(String),
    LBrace,
    RBrace,
    Semicolon,
}

pub fn tokenize(input: &str) -> Vec<Token> {
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
            c if c.is_whitespace() => {
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