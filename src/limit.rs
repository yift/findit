use crate::{cli_args::CliArgs, file_wrapper::FileWrapper, walker::Walk};

#[derive(Debug)]
struct Limit {
    limit: usize,
    counter: usize,
}

impl Walk for Limit {
    fn step(&mut self, file: &FileWrapper) {
        self.counter += 1;
        if self.counter >= self.limit {
            file.debugger().log(&|| {
                format!(
                    "Limit of {} reached after processing file: {}",
                    self.limit,
                    file.path().display()
                )
            });
        }
    }
    fn enough(&self) -> bool {
        self.counter >= self.limit
    }
}
pub(crate) fn make_limit(args: &CliArgs) -> Option<Box<dyn Walk>> {
    let limit = args.limit?;
    Some(Box::new(Limit { limit, counter: 0 }))
}

#[cfg(test)]
mod tests {
    use std::{fs, rc::Rc};

    use clap::Parser;

    use crate::{
        cli_args::CliArgs, debugger::create_debugger, errors::FindItError,
        file_wrapper::FileWrapper, limit::make_limit,
    };

    #[test]
    fn test_limit_with_debugger() -> Result<(), FindItError> {
        let temp_dir = tempfile::tempdir()?;
        let log_path = temp_dir
            .path()
            .join("limit/debug/directory")
            .join("debug.log");
        let debugger = Rc::new(create_debugger(Some(&log_path))?);

        let args = CliArgs::parse_from(vec!["findit", "--limit", "2"]);

        let mut step = make_limit(&args).unwrap();

        step.step(&FileWrapper::new_with_debugger(
            temp_dir.path().join("file1.txt"),
            0,
            &debugger,
        ));
        assert!(!step.enough());

        step.step(&FileWrapper::new_with_debugger(
            temp_dir.path().join("file2.txt"),
            0,
            &debugger,
        ));
        assert!(step.enough());

        drop(debugger);

        let log_contents = fs::read_to_string(&log_path)?;
        assert!(log_contents.contains("Limit of 2 reached after processing file:"));
        Ok(())
    }
}
