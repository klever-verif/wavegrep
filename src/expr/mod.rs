#![allow(dead_code)]

pub mod eval;
pub mod lexer;
pub mod parser;

use crate::error::WavepeekError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expression {
    source: String,
}

impl Expression {
    pub fn source(&self) -> &str {
        &self.source
    }
}

pub fn parse(source: &str) -> Result<Expression, WavepeekError> {
    parser::parse(source)
}
