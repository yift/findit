use std::{fs, path::PathBuf, rc::Rc};

use crate::{
    cli_args::CliArgs,
    debugger::{Debugger, create_debugger},
    errors::FindItError,
    file_wrapper::FileWrapper,
};

#[derive(Debug)]
pub(crate) struct Walker {
    root: PathBuf,
    depth: usize,
    node_first: bool,
    max_depth: Option<usize>,
    debugger: Rc<Box<dyn Debugger>>,
}
pub(crate) trait Walk {
    fn step(&mut self, file: &FileWrapper);
    fn enough(&self) -> bool;
}
impl Walker {
    pub(crate) fn walk(&self, stepper: &mut Box<dyn Walk>) -> Result<(), FindItError> {
        if stepper.enough() {
            return Ok(());
        }
        if !self.node_first {
            stepper.step(&FileWrapper::new_with_debugger(
                self.root.clone(),
                self.depth,
                &self.debugger,
            ));
        }

        if self.depth < self.max_depth.unwrap_or(usize::MAX) && self.root.is_dir() {
            self.debugger.log(&|| {
                format!(
                    "Walking into directory: [{}] at depth: {}",
                    self.root.display(),
                    self.depth
                )
            });
            let paths = fs::read_dir(&self.root)?;
            for path in paths {
                let path = path?;
                let walker = Walker {
                    depth: self.depth + 1,
                    root: path.path(),
                    node_first: self.node_first,
                    max_depth: self.max_depth,
                    debugger: self.debugger.clone(),
                };
                walker.walk(stepper)?;
            }
        }

        if self.node_first && !stepper.enough() {
            stepper.step(&FileWrapper::new_with_debugger(
                self.root.clone(),
                self.depth,
                &self.debugger,
            ));
        }

        Ok(())
    }
}
impl TryFrom<&CliArgs> for Walker {
    type Error = FindItError;
    fn try_from(value: &CliArgs) -> Result<Self, Self::Error> {
        let root = match &value.root {
            Some(path) => path.clone(),
            None => PathBuf::from("."),
        };
        let debugger = create_debugger(value.debug_output_file.as_ref())?;
        if root.exists() {
            Ok(Walker {
                root,
                depth: 0,
                node_first: value.node_first,
                max_depth: value.max_depth,
                debugger: Rc::new(debugger),
            })
        } else {
            Err(FindItError::NoSuchFile(root))
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;

    #[test]
    fn try_from_nop_such_file() {
        let args = CliArgs::parse_from(vec!["-", "foo/bar/no/such/file"]);

        let err = Walker::try_from(&args).err();

        assert!(err.is_some())
    }
}
