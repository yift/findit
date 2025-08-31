use std::collections::VecDeque;

use crate::{
    errors::FindItError,
    expr::Evaluator,
    file_wrapper::FileWrapper,
    functions::spawn::execute::Executor,
    value::{Value, ValueType},
};

struct Fire {
    executor: Executor,
}

impl Evaluator for Fire {
    fn expected_type(&self) -> ValueType {
        ValueType::Number
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Some(mut command) = self.executor.execute(file) else {
            return Value::Empty;
        };
        let Ok(result) = command.spawn() else {
            return Value::Empty;
        };

        result.id().into()
    }
}
pub(crate) fn build_fire(
    mut args: VecDeque<Box<dyn Evaluator>>,
    into: bool,
) -> Result<Box<dyn Evaluator>, FindItError> {
    let Some(exec) = args.pop_front() else {
        return Err(FindItError::BadExpression(
            "FIRE must have at least one argument.".into(),
        ));
    };
    if exec.expected_type() != ValueType::String && exec.expected_type() != ValueType::Path {
        return Err(FindItError::BadExpression(
            "Can only execute string or files.".into(),
        ));
    }
    let into = if into {
        let Some(into) = args.pop_back() else {
            return Err(FindItError::BadExpression(
                "Fire into last argument must be present.".into(),
            ));
        };
        if into.expected_type() != ValueType::String && into.expected_type() != ValueType::Path {
            return Err(FindItError::BadExpression(
                "Can only fire into string or file.".into(),
            ));
        }

        Some(into)
    } else {
        None
    };
    let executor = Executor::new(exec, args, into);
    Ok(Box::new(Fire { executor }))
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::{env, fs, os::unix::fs::OpenOptionsExt, path::Path, thread::sleep, time::Duration};

    use tempfile::tempdir;

    use crate::{
        errors::FindItError,
        expr::read_expr,
        file_wrapper::FileWrapper,
        value::{Value, ValueType},
    };

    #[test]
    fn test_fire_with_no_args() {
        let sql = "fire()";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }

    #[test]
    fn test_fire_into_no_into() {
        let sql = "fire_into('rm')";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }

    #[test]
    fn test_fire_into_into_bool() {
        let sql = "fire_into('rm', FALSE)";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }

    #[test]
    fn test_fire_numeric_arg() {
        let sql = "fire(12)";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }

    #[test]
    fn test_fire_expected_return() {
        let sql = "fire(path)";
        let expr = read_expr(sql).unwrap();

        assert_eq!(expr.expected_type(), ValueType::Number);
    }

    #[test]
    fn test_fire_non_exec_returns_empty() -> Result<(), FindItError> {
        let sql = "fire(path)";
        let expr = read_expr(sql)?;
        let file = env::current_dir()?;
        let wrapper = FileWrapper::new(file, 1);

        let value = expr.eval(&wrapper);

        assert_eq!(value, Value::Empty);

        Ok(())
    }

    #[test]
    fn test_fire_null_empty() -> Result<(), FindItError> {
        let sql = "fire(parent)";
        let expr = read_expr(sql)?;
        let file = Path::new("/").to_path_buf();
        let wrapper = FileWrapper::new(file, 1);

        let value = expr.eval(&wrapper);

        assert_eq!(value, Value::Empty);

        Ok(())
    }

    #[test]
    fn test_fire_execute() -> Result<(), FindItError> {
        let dir = tempdir()?;
        let path = dir.path();
        if !path.exists() {
            panic!("Path should exists now");
        }

        let sql = "fire('rm', '-rf', path)";
        let expr = read_expr(sql)?;
        let wrapper = FileWrapper::new(path.to_path_buf(), 1);

        let value = expr.eval(&wrapper);

        match value {
            Value::Number(_) => {}
            _ => {
                panic!("Expecting pid");
            }
        }

        for _ in 0..2000 {
            if !path.exists() {
                return Ok(());
            }
            sleep(Duration::from_millis(10));
        }
        panic!("File exists");
    }

    #[test]
    fn test_fire_execute_path() -> Result<(), FindItError> {
        let dir = tempdir()?;
        let path = dir.path();
        let bash_file = path.join("exec").join("test");
        let bash_dir = bash_file.parent().unwrap();
        fs::create_dir_all(bash_dir).ok();
        let mut file = fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .mode(0o775)
            .open(&bash_file)?;
        let file_to_create = path.join("file");
        fs::remove_file(&file_to_create).ok();
        writeln!(&mut file, "#!/bin/bash")?;
        writeln!(&mut file)?;
        writeln!(
            &mut file,
            "echo 'text' > {}",
            file_to_create.to_str().unwrap_or_default()
        )?;
        drop(file);

        let sql = "fire(#test)";
        let expr = read_expr(sql)?;
        let wrapper = FileWrapper::new(bash_dir.to_path_buf(), 1);

        let value = expr.eval(&wrapper);

        match value {
            Value::Number(_) => {}
            _ => {
                panic!("Expecting pid");
            }
        }

        for _ in 0..2000 {
            if file_to_create.exists() {
                return Ok(());
            }
            sleep(Duration::from_millis(10));
        }
        panic!("File was not created");
    }

    #[test]
    fn test_fire_into() -> Result<(), FindItError> {
        let dir = tempdir()?;
        let path = dir.path();
        let bash_file = path.join("exec").join("test");
        let bash_dir = bash_file.parent().unwrap();
        fs::create_dir_all(bash_dir).ok();
        let mut file = fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .mode(0o775)
            .open(&bash_file)?;
        let file_to_create = path.join("dir").join("file");
        fs::remove_file(&file_to_create).ok();
        writeln!(&mut file, "#!/bin/bash")?;
        writeln!(&mut file)?;
        writeln!(&mut file, "echo 'text'")?;
        drop(file);

        let sql = format!("fire_into(#test, '{}')", file_to_create.to_str().unwrap());
        let expr = read_expr(&sql)?;
        let wrapper = FileWrapper::new(bash_dir.to_path_buf(), 1);

        let value = expr.eval(&wrapper);

        match value {
            Value::Number(_) => {}
            _ => {
                panic!("Expecting pid");
            }
        }

        for _ in 0..2000 {
            if file_to_create.exists()
                && fs::read_to_string(&file_to_create).ok() == Some("text\n".into())
            {
                return Ok(());
            }
            sleep(Duration::from_millis(10));
        }
        panic!("File was not created");
    }

    #[test]
    fn test_fire_into_append() -> Result<(), FindItError> {
        let dir = tempdir()?;
        let path = dir.path();
        let bash_file = path.join("exec").join("test");
        let bash_dir = bash_file.parent().unwrap();
        fs::create_dir_all(bash_dir).ok();
        let mut file = fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .mode(0o775)
            .open(&bash_file)?;
        let file_to_create = path.join("dir").join("file");

        fs::create_dir_all(file_to_create.parent().unwrap())?;
        let mut start_with = fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&file_to_create)?;
        writeln!(&mut start_with, "line 1")?;
        drop(start_with);

        writeln!(&mut file, "#!/bin/bash")?;
        writeln!(&mut file)?;
        writeln!(&mut file, "echo 'line 2'")?;
        drop(file);

        let sql = "fire_into(#test, parent / 'dir' / 'file')";
        let expr = read_expr(sql)?;
        let wrapper = FileWrapper::new(bash_dir.to_path_buf(), 1);

        let value = expr.eval(&wrapper);

        match value {
            Value::Number(_) => {}
            _ => {
                panic!("Expecting pid");
            }
        }

        for _ in 0..2000 {
            if file_to_create.exists()
                && fs::read_to_string(&file_to_create).ok() == Some("line 1\nline 2\n".into())
            {
                return Ok(());
            }
            sleep(Duration::from_millis(10));
        }
        panic!("File was not appended");
    }

    #[test]
    fn test_fire_into_non_into_returns_empty() -> Result<(), FindItError> {
        let sql = "fire_into('echo', parent)";
        let expr = read_expr(sql)?;
        let file = Path::new("/");
        let wrapper = FileWrapper::new(file.to_path_buf(), 1);

        let value = expr.eval(&wrapper);

        assert_eq!(value, Value::Empty);

        Ok(())
    }

    #[test]
    fn test_fire_into_bad_location_returns_empty() -> Result<(), FindItError> {
        let sql = "fire_into('echo', '/bin/not/a/valid/location/a.txt')";
        let expr = read_expr(sql)?;
        let file = Path::new("/");
        let wrapper = FileWrapper::new(file.to_path_buf(), 1);

        let value = expr.eval(&wrapper);

        assert_eq!(value, Value::Empty);

        Ok(())
    }
}
