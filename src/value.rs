use std::{
    ffi::OsStr,
    fmt::Display,
    path::{Path, PathBuf},
    rc::Rc,
    time::SystemTime,
};

use chrono::{DateTime, Local};

use crate::{
    class_type::{Class, ClassType},
    lazy_list::LazyList,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub(crate) enum Value {
    String(String),
    Path(PathBuf),
    Number(u64),
    Bool(bool),
    Date(DateTime<Local>),
    List(List),
    Class(Class),
    Empty,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub(crate) struct List {
    items: LazyList<Value>,
    item_type: Rc<ValueType>,
}

impl List {
    pub(crate) fn new_lazy(
        item_type: Rc<ValueType>,
        items: impl Iterator<Item = Value> + 'static,
    ) -> Self {
        let items: Box<dyn Iterator<Item = Value>> = Box::new(items);
        let items = items.into();

        Self { items, item_type }
    }
    pub(crate) fn new_eager(item_type: Rc<ValueType>, items: impl Iterator<Item = Value>) -> Self {
        let items = items.collect::<Vec<_>>();
        Self::new_from_vec(item_type, items)
    }
    pub(crate) fn new_from_vec(item_type: Rc<ValueType>, items: Vec<Value>) -> Self {
        let items = items.into();

        Self { items, item_type }
    }
    pub(crate) fn has_items(self) -> bool {
        self.items.into_iter().next().is_some()
    }
    pub(crate) fn count(self) -> usize {
        self.items.into_iter().count()
    }
    pub(crate) fn items(self) -> LazyList<Value> {
        self.items
    }
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

impl From<u32> for Value {
    fn from(value: u32) -> Self {
        Value::Number(value as u64)
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
            Value::List(lst) => write!(f, "{}", lst.items),
            Value::Class(cls) => write!(f, "{}", cls),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub(crate) enum ValueType {
    Bool,
    Number,
    Path,
    String,
    Date,
    List(Rc<ValueType>),
    Class(Rc<ClassType>),
    Empty,
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
            ValueType::List(tp) => write!(f, "list<{tp}>"),
            ValueType::Class(tp) => tp.fmt(f),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::errors::FindItError;

    use super::*;

    #[test]
    fn test_display_value_type() -> Result<(), FindItError> {
        assert_eq!(ValueType::Bool.to_string(), "boolean");
        assert_eq!(ValueType::Number.to_string(), "number");
        assert_eq!(ValueType::Path.to_string(), "path");
        assert_eq!(ValueType::String.to_string(), "string");
        assert_eq!(ValueType::Date.to_string(), "date");
        assert_eq!(ValueType::Empty.to_string(), "empty");
        assert_eq!(
            ValueType::List(Rc::new(ValueType::Path)).to_string(),
            "list<path>"
        );
        assert_eq!(
            ValueType::Class(Rc::new(ClassType::new(&[]))).to_string(),
            "class<>"
        );
        Ok(())
    }
}
