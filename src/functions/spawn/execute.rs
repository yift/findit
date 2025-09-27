use std::{
    fs::{self, OpenOptions},
    path::Path,
    process::Command,
};

use crate::{expr::Evaluator, file_wrapper::FileWrapper, value::Value};

pub(crate) struct Executor {
    exec: Box<dyn Evaluator>,
    args: Vec<Box<dyn Evaluator>>,
    into: Option<Box<dyn Evaluator>>,
}

impl Executor {
    pub(crate) fn new(
        exec: Box<dyn Evaluator>,
        args: Vec<Box<dyn Evaluator>>,
        into: Option<Box<dyn Evaluator>>,
    ) -> Self {
        Self { exec, args, into }
    }

    pub(crate) fn execute(&self, file: &FileWrapper) -> Option<Command> {
        let mut command = match &self.exec.eval(file) {
            Value::String(str) => Command::new(str),
            Value::Path(path) => Command::new(path),
            _ => {
                return None;
            }
        };

        self.add_args(&mut command, file);

        command = self.add_into(command, file)?;

        Some(command)
    }

    fn add_args(&self, command: &mut Command, file: &FileWrapper) {
        for arg in &self.args {
            let arg = arg.eval(file).to_string();
            command.arg(arg);
        }
    }

    fn add_into(&self, mut command: Command, file: &FileWrapper) -> Option<Command> {
        let Some(into) = &self.into else {
            return Some(command);
        };
        let path = match into.eval(file) {
            Value::String(str) => Path::new(&str).to_path_buf(),
            Value::Path(path) => path.to_path_buf(),
            _ => {
                return Option::None;
            }
        };
        let parent = path.parent()?;

        if !parent.exists() && fs::create_dir_all(parent).is_err() {
            return Option::None;
        }
        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(path)
            .ok()?;
        command.stdout(file);
        Some(command)
    }
}
