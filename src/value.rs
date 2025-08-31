use std::{
    ffi::OsStr,
    fmt::Display,
    path::{Path, PathBuf},
    time::SystemTime,
};

use chrono::{DateTime, Local};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub(crate) enum Value {
    Empty,
    String(String),
    Path(PathBuf),
    Number(u64),
    Bool(bool),
    Date(DateTime<Local>),
}

impl From<&Path> for Value {
    fn from(value: &Path) -> Self {
        Value::Path(value.to_path_buf())
    }
}
impl From<PathBuf> for Value {
    fn from(value: PathBuf) -> Self {
        Value::Path(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::String(value.to_string())
    }
}
impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
    }
}
impl From<&OsStr> for Value {
    fn from(value: &OsStr) -> Self {
        value.to_str().into()
    }
}

impl From<usize> for Value {
    fn from(value: usize) -> Self {
        Value::Number(value as u64)
    }
}

impl From<u64> for Value {
    fn from(value: u64) -> Self {
        Value::Number(value)
    }
}

impl From<DateTime<Local>> for Value {
    fn from(value: DateTime<Local>) -> Self {
        Value::Date(value)
    }
}
impl From<SystemTime> for Value {
    fn from(value: SystemTime) -> Self {
        let date: DateTime<Local> = value.into();
        date.into()
    }
}
impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

impl<T: Into<Value>> From<Option<T>> for Value {
    fn from(value: Option<T>) -> Self {
        match value {
            None => Value::Empty,
            Some(t) => t.into(),
        }
    }
}

impl<T: Into<Value>, E> From<Result<T, E>> for Value {
    fn from(value: Result<T, E>) -> Self {
        value.ok().into()
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Empty => Ok(()),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Number(n) => write!(f, "{n}"),
            Value::Path(p) => write!(f, "{}", p.as_os_str().to_str().unwrap_or_default()),
            Value::String(s) => write!(f, "{s}"),
            Value::Date(dt) => write!(f, "{}", dt.format("%d/%b/%Y %H:%M:%S")),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub(crate) enum ValueType {
    Empty,
    Bool,
    Number,
    Path,
    String,
    Date,
}

impl Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueType::Empty => "empty".fmt(f),
            ValueType::Bool => "boolean".fmt(f),
            ValueType::Date => "date".fmt(f),
            ValueType::Number => "number".fmt(f),
            ValueType::Path => "path".fmt(f),
            ValueType::String => "string".fmt(f),
        }
    }
}
