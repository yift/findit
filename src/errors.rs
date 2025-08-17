use sqlparser::parser::ParserError;
use std::{io::Error as IoError, num::ParseIntError, path::PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FindItError {
    #[error("SQL Parse error: `{0}`")]
    SqlParserError(#[from] ParserError),
    #[error("Number Parse error: `{0}`")]
    IntParserError(#[from] ParseIntError),
    #[error("IO Error: `{0}`")]
    IoError(#[from] IoError),
    #[error("No such file: `{0}`")]
    NoSuchFile(PathBuf),
    #[error("Bad filter: `{0}`")]
    BadFilter(String),
    #[error("Bad expression: `{0}`")]
    BadExpression(String),
    #[error("Could not parse `{0}` because : `{0}`")]
    DisplayParserError(String, String),
}
