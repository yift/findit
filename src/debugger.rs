use std::fmt::Debug;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use crate::errors::FindItError;

pub(crate) trait Debugger: Debug {
    fn log(&self, f: &dyn Fn() -> String);
}

#[derive(Debug)]
struct EmptyDebugger;
impl Debugger for EmptyDebugger {
    fn log(&self, _f: &dyn Fn() -> String) {}
}

#[derive(Debug)]
struct FileDebugger {
    file: File,
}

impl Debugger for FileDebugger {
    fn log(&self, f: &dyn Fn() -> String) {
        let mut file = &self.file;
        let msg = f();
        writeln!(file, "{}", msg).ok();
    }
}
pub(crate) fn create_debugger(path: Option<&PathBuf>) -> Result<Box<dyn Debugger>, FindItError> {
    if let Some(p) = path {
        fs::create_dir_all(p.parent().unwrap())?;
        let file = File::create(p)?;
        Ok(Box::new(FileDebugger { file }))
    } else {
        Ok(Box::new(EmptyDebugger))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::errors::FindItError;

    #[test]
    fn test_file_debug() -> Result<(), FindItError> {
        let temp_dir = tempfile::tempdir()?;
        let log_path = temp_dir.path().join("directory").join("debug.log");
        let debugger = super::create_debugger(Some(&log_path))?;

        debugger.log(&|| "This is a test log entry.".to_string());
        debugger.log(&|| "Logging another entry.".to_string());

        drop(debugger);

        let log_contents = fs::read_to_string(&log_path)?;
        let expected_contents = "This is a test log entry.\nLogging another entry.\n";
        assert_eq!(log_contents, expected_contents);
        Ok(())
    }
}
