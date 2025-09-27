use std::{io::Error as IoError, num::ParseIntError, path::PathBuf};
use thiserror::Error;

use crate::parser::expression::ParserError;

#[derive(Error, Debug)]
pub enum FindItError {
    #[error("Number Parse error: `{0}`")]
    IntParserError(#[from] ParseIntError),
    #[error("IO Error: `{0}`")]
    IoError(#[from] IoError),
    #[error("No such file: `{0}`")]
    NoSuchFile(PathBuf),
    #[error("Bad filter: `{0}`")]
    BadFilter(String),
    #[error("Bad order by: `{0}`")]
    BadOrderBy(String),
    #[error("Bad expression: `{0}`")]
    BadExpression(String),
    #[error("Could not parse `{0}` because : `{0}`")]
    DisplayParserError(String, String),
    #[error("Expression parse error: `{0}`")]
    ParserError(#[from] ParserError),
}
