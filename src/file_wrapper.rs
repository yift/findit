use std::{
    fmt::{Debug, Display},
    fs,
    ops::Deref,
    path::PathBuf,
    rc::Rc,
};

use crate::{errors::FindItError, value::Value};

#[derive(Debug, Clone)]
pub(crate) struct FileWrapper {
    path: PathBuf,
    depth: usize,
    bindings: Vec<Rc<Value>>,
}
impl FileWrapper {
    pub(crate) fn new(path: PathBuf, depth: usize) -> Self {
        Self {
            path,
            depth,
            bindings: Vec::new(),
        }
    }

    pub(crate) fn with_binding(&self, binding: Value) -> Self {
        let mut new_binding = self.bindings.clone();
        new_binding.push(Rc::new(binding));
        Self {
            path: self.path.to_path_buf(),
            depth: self.depth,
            bindings: new_binding,
        }
    }

    pub(crate) fn dept(&self) -> usize {
        self.depth
    }

    pub(crate) fn path(&self) -> &PathBuf {
        &self.path
    }

    pub(crate) fn read(&self) -> Result<String, FindItError> {
        let string = fs::read_to_string(&self.path)?;
        Ok(string)
    }

    pub(crate) fn get_binding(&self, index: usize) -> Value {
        self.bindings
            .get(index)
            .map(|b| b.deref().clone())
            .unwrap_or(Value::Empty)
    }

    pub(crate) fn count(&self) -> Result<usize, FindItError> {
        if !self.path.exists() {
            return Ok(0);
        }
        if !self.path.is_dir() {
            return Ok(1);
        }
        let paths = fs::read_dir(&self.path)?;
        Ok(paths.count())
    }
}
impl Display for FileWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path.as_os_str().to_str().unwrap_or_default())
    }
}
