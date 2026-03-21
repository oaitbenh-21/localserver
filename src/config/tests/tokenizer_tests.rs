use crate::config::tokenizer::{Token, tokenize};

// ── Basic tokens ──────────────────────────────────────────────────────────────

#[test]
fn test_empty_input() {
    let tokens = tokenize("");
    assert!(tokens.is_empty());
}

#[test]
fn test_single_word() {
    let tokens = tokenize("server");
    assert_eq!(tokens, vec![Token::Word("server".to_string())]);
}

#[test]
fn test_braces() {
    let tokens = tokenize("{}");
    assert_eq!(tokens, vec![Token::LBrace, Token::RBrace]);
}

#[test]
fn test_semicolon() {
    let tokens = tokenize(";");
    assert_eq!(tokens, vec![Token::Semicolon]);
}

#[test]
fn test_word_with_semicolon() {
    let tokens = tokenize("host 127.0.0.1;");
    assert_eq!(
        tokens,
        vec![
            Token::Word("host".to_string()),
            Token::Word("127.0.0.1".to_string()),
            Token::Semicolon,
        ]
    );
}

// ── Whitespace handling ───────────────────────────────────────────────────────

#[test]
fn test_multiple_spaces_between_words() {
    let tokens = tokenize("host    127.0.0.1");
    assert_eq!(
        tokens,
        vec![
            Token::Word("host".to_string()),
            Token::Word("127.0.0.1".to_string()),
        ]
    );
}

#[test]
fn test_newlines_are_whitespace() {
    let tokens = tokenize("host\n127.0.0.1");
    assert_eq!(
        tokens,
        vec![
            Token::Word("host".to_string()),
            Token::Word("127.0.0.1".to_string()),
        ]
    );
}

#[test]
fn test_tabs_are_whitespace() {
    let tokens = tokenize("host\t127.0.0.1");
    assert_eq!(
        tokens,
        vec![
            Token::Word("host".to_string()),
            Token::Word("127.0.0.1".to_string()),
        ]
    );
}

#[test]
fn test_leading_trailing_whitespace() {
    let tokens = tokenize("  server  ");
    assert_eq!(tokens, vec![Token::Word("server".to_string())]);
}

// ── Real config fragments ─────────────────────────────────────────────────────

#[test]
fn test_server_block_tokens() {
    let input = "server {\n    host 127.0.0.1;\n}";
    let tokens = tokenize(input);
    assert_eq!(
        tokens,
        vec![
            Token::Word("server".to_string()),
            Token::LBrace,
            Token::Word("host".to_string()),
            Token::Word("127.0.0.1".to_string()),
            Token::Semicolon,
            Token::RBrace,
        ]
    );
}

#[test]
fn test_location_block_tokens() {
    let input = "location / {\n    root ./www;\n}";
    let tokens = tokenize(input);
    assert_eq!(
        tokens,
        vec![
            Token::Word("location".to_string()),
            Token::Word("/".to_string()),
            Token::LBrace,
            Token::Word("root".to_string()),
            Token::Word("./www".to_string()),
            Token::Semicolon,
            Token::RBrace,
        ]
    );
}
