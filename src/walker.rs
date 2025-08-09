use std::{env, fs, path::PathBuf};

use crate::{cli_args::CliArgs, errors::FindItError, file_wrapper::FileWrapper};

#[derive(Debug)]
pub(crate) struct Walker {
    root: PathBuf,
    depth: usize,
    node_first: bool,
    max_depth: Option<usize>,
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
            stepper.step(&FileWrapper::new(self.root.clone(), self.depth))
        }

        if self.depth < self.max_depth.unwrap_or(usize::MAX) && self.root.is_dir() {
            let paths = fs::read_dir(&self.root)?;
            for path in paths {
                let path = path?;
                let walker = Walker {
                    depth: self.depth + 1,
                    root: path.path(),
                    node_first: self.node_first,
                    max_depth: self.max_depth,
                };
                walker.walk(stepper)?;
            }
        }

        if self.node_first {
            stepper.step(&FileWrapper::new(self.root.clone(), self.depth))
        }

        Ok(())
    }
}
impl TryFrom<&CliArgs> for Walker {
    type Error = FindItError;
    fn try_from(value: &CliArgs) -> Result<Self, Self::Error> {
        let root = match &value.root {
            Some(path) => path.clone(),
            None => env::current_dir()?,
        };
        if !root.exists() {
            Err(FindItError::NoSuchFile(root))
        } else {
            Ok(Walker {
                root,
                depth: 0,
                node_first: value.node_first,
                max_depth: value.max_depth,
            })
        }
    }
}
