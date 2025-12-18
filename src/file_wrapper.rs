use std::{
    fmt::{Debug, Display},
    fs,
    ops::Deref,
    path::PathBuf,
    rc::Rc,
};

use crate::{debugger::Debugger, errors::FindItError, value::Value};

#[derive(Debug, Clone)]
pub(crate) struct FileWrapper {
    path: PathBuf,
    depth: usize,
    bindings: Vec<Rc<Value>>,
    debugger: Rc<Box<dyn Debugger>>,
}
impl FileWrapper {
    pub(crate) fn new_with_debugger(
        path: PathBuf,
        depth: usize,
        debugger: &Rc<Box<dyn Debugger>>,
    ) -> Self {
        Self {
            path,
            depth,
            bindings: Vec::new(),
            debugger: debugger.clone(),
        }
    }

    pub(crate) fn with_file(&self, path: PathBuf) -> Self {
        Self {
            path,
            depth: self.depth + 1,
            bindings: self.bindings.clone(),
            debugger: self.debugger.clone(),
        }
    }

    pub(crate) fn with_binding(&self, binding: Value) -> Self {
        let mut new_binding = self.bindings.clone();
        new_binding.push(Rc::new(binding));
        Self {
            path: self.path.to_path_buf(),
            depth: self.depth,
            bindings: new_binding,
            debugger: self.debugger.clone(),
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

    pub(crate) fn debugger(&self) -> &Rc<Box<dyn Debugger>> {
        &self.debugger
    }
}
impl Display for FileWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path.as_os_str().to_str().unwrap_or_default())
    }
}

#[cfg(test)]
impl FileWrapper {
    pub(crate) fn new(path: PathBuf, depth: usize) -> Self {
        use crate::debugger;

        let debugger = debugger::create_debugger(None).unwrap();
        Self {
            path,
            depth,
            bindings: Vec::new(),
            debugger: Rc::new(debugger),
        }
    }
}
