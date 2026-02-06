#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Identifier(String),
    Literal(String),
    Operator(String),
    LeftParen,
    RightParen,
}

pub fn tokenize(_source: &str) -> Vec<Token> {
    Vec::new()
}
