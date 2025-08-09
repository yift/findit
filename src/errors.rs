use sqlparser::parser::ParserError;
use std::{io::Error as IoError, path::PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FindItError {
    #[error("Parse error: `{0}`")]
    ParserError(#[from] ParserError),
    #[error("IO Error: `{0}`")]
    IoError(#[from] IoError),
    #[error("No such file: `{0}`")]
    NoSuchFile(PathBuf),
}
