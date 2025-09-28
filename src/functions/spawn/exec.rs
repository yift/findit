use std::collections::VecDeque;

use crate::{
    errors::FindItError,
    expr::Evaluator,
    file_wrapper::FileWrapper,
    functions::spawn::execute::Executor,
    value::{Value, ValueType},
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub(super) enum ExecType {
    Status,
    IntoStatus,
    CaptureOutput,
}
struct Execute {
    executor: Executor,
    exec_type: ExecType,
}

impl Evaluator for Execute {
    fn expected_type(&self) -> ValueType {
        if self.exec_type == ExecType::CaptureOutput {
            ValueType::String
        } else {
            ValueType::Bool
        }
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Some(mut command) = self.executor.execute(file) else {
            return Value::Empty;
        };
        if self.exec_type == ExecType::CaptureOutput {
            let Some(output) = command.output().ok() else {
                return Value::Empty;
            };
            String::from_utf8(output.stdout).into()
        } else {
            let Some(status) = command.status().ok() else {
                return Value::Empty;
            };
            status.success().into()
        }
    }
}

pub(crate) fn build_capture_output_exec(
    mut args: VecDeque<Box<dyn Evaluator>>,
) -> Result<Box<dyn Evaluator>, FindItError> {
    let Some(exec) = args.pop_front() else {
        return Err(FindItError::BadExpression(
            "EXEC must have at least one argument.".into(),
        ));
    };
    if exec.expected_type() != ValueType::String && exec.expected_type() != ValueType::Path {
        return Err(FindItError::BadExpression(
            "Can only execute string or files.".into(),
        ));
    }
    let into = None;
    let executor = Executor::new(exec, args.into(), into);
    Ok(build_exec(executor, ExecType::CaptureOutput))
}

pub(super) fn build_exec(executor: Executor, exec_type: ExecType) -> Box<dyn Evaluator> {
    Box::new(Execute {
        executor,
        exec_type,
    })
}

#[cfg(test)]
mod tests {

    use std::{env, fs, path::Path};

    use tempfile::tempdir;

    use crate::{
        errors::FindItError,
        expr::read_expr,
        file_wrapper::FileWrapper,
        value::{Value, ValueType},
    };

    #[test]
    fn test_exec_with_no_args() {
        let sql = "exec()";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }

    #[test]
    fn test_exec_into_number() {
        let sql = "exec('rm' into 4000)";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }

    #[test]
    fn test_exec_out_with_numeric_arg() {
        let sql = "execOut(32)";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }

    #[test]
    fn test_exec_expected_return() {
        let sql = "exec(parent)";
        let expr = read_expr(sql).unwrap();

        assert_eq!(expr.expected_type(), ValueType::Bool);
    }

    #[test]
    fn test_exec_out_expected_return() {
        let sql = "execOut('echo')";
        let expr = read_expr(sql).unwrap();

        assert_eq!(expr.expected_type(), ValueType::String);
    }

    #[test]
    fn test_exec_success_return_true() -> Result<(), FindItError> {
        let sql = "exec('ls', path)";
        let expr = read_expr(sql).unwrap();
        let file = env::current_dir()?;
        let wrapper = FileWrapper::new(file, 1);

        let value = expr.eval(&wrapper);

        assert_eq!(value, Value::Bool(true));
        Ok(())
    }

    #[test]
    fn test_exec_fail_return_false() -> Result<(), FindItError> {
        let sql = "exec('ls', '/bin/no/such/dir/')";
        let expr = read_expr(sql).unwrap();
        let file = env::current_dir()?;
        let wrapper = FileWrapper::new(file, 1);

        let value = expr.eval(&wrapper);

        assert_eq!(value, Value::Bool(false));
        Ok(())
    }

    #[test]
    fn test_exec_out_return_results() -> Result<(), FindItError> {
        let sql = "exec_out('cat', path)";
        let expr = read_expr(sql).unwrap();
        let file =
            Path::new("tests/test_cases/display/test_files/thing/good-581.txt").to_path_buf();
        let expected_text = fs::read_to_string(&file)?;
        let wrapper = FileWrapper::new(file, 1);

        let value = expr.eval(&wrapper);

        assert_eq!(value, Value::String(expected_text));
        Ok(())
    }

    #[test]
    fn test_exec_into_return_true() -> Result<(), FindItError> {
        let dir = tempdir()?;
        let out_file = dir.path().join("out.txt");

        let sql = format!("exec('cat', path into '{}')", out_file.to_str().unwrap());
        let expr = read_expr(&sql).unwrap();
        let file =
            Path::new("tests/test_cases/display/test_files/thing/good-581.txt").to_path_buf();
        let expected_text = fs::read_to_string(&file)?;
        let wrapper = FileWrapper::new(file, 1);

        let value = expr.eval(&wrapper);

        assert_eq!(value, Value::Bool(true));
        let txt = fs::read_to_string(out_file)?;
        assert_eq!(expected_text, txt);
        Ok(())
    }

    #[test]
    fn test_exec_out_with_no_arg() {
        let sql = "execOut()";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }

    #[test]
    fn test_exec_nothing_if_nothing_to_execute() -> Result<(), FindItError> {
        let sql = "exec(content)";
        let expr = read_expr(sql)?;
        let file = Path::new("no/such/file/text.txt").to_path_buf();
        let wrapper = FileWrapper::new(file, 1);

        let value = expr.eval(&wrapper);

        assert_eq!(value, Value::Empty);
        Ok(())
    }

    #[test]
    fn test_exec_bad_command() -> Result<(), FindItError> {
        let sql = "exec(\"nothing_to_run\")";
        let expr = read_expr(sql)?;
        let file = Path::new("no/such/file/text.txt").to_path_buf();
        let wrapper = FileWrapper::new(file, 1);

        let value = expr.eval(&wrapper);

        assert_eq!(value, Value::Empty);
        Ok(())
    }

    #[test]
    fn test_exec_out_bad_command() -> Result<(), FindItError> {
        let sql = "execOut(\"nothing_to_run\")";
        let expr = read_expr(sql)?;
        let file = Path::new("no/such/file/text.txt").to_path_buf();
        let wrapper = FileWrapper::new(file, 1);

        let value = expr.eval(&wrapper);

        assert_eq!(value, Value::Empty);
        Ok(())
    }
}
