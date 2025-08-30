use std::{
    fmt::{Debug, Display},
    fs,
    path::PathBuf,
};

use crate::errors::FindItError;

#[derive(Debug, Clone)]
pub(crate) struct FileWrapper {
    path: PathBuf,
    depth: usize,
}
impl FileWrapper {
    pub(crate) fn new(path: PathBuf, depth: usize) -> Self {
        Self { path, depth }
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
