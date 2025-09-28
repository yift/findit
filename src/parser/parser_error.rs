use thiserror::Error;

use crate::parser::{lexer::LexerError, span::Span};

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Lexer error: `{0}`")]
    LexerError(#[from] LexerError),
    #[error("Unexpected end of expression")]
    UnexpectedEof,
    #[error("Unexpected token at `{0}`")]
    UnexpectedToken(Span),
    #[error("Case without any branches `{0}`")]
    NoBranches(Span),
}
