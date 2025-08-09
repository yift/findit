use std::{
    fmt::{Debug, Display},
    path::PathBuf,
};

#[derive(Debug)]
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
}
impl Display for FileWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path.as_os_str().to_str().unwrap_or_default())
    }
}
