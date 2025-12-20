use std::fs::{self, ReadDir};
use std::path::PathBuf;
use std::rc::Rc;

use crate::{
    errors::FindItError,
    evaluators::expr::Evaluator,
    file_wrapper::FileWrapper,
    value::{List, Value, ValueType},
};

struct Walker {
    stack: Vec<ReadDir>,
}

impl Walker {
    fn new(path: PathBuf) -> Self {
        let stack = match fs::read_dir(path) {
            Ok(rd) => vec![rd],
            Err(_) => vec![],
        };
        Self { stack }
    }
}
impl Iterator for Walker {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(top) = self.stack.last_mut() {
            match top.next().and_then(Result::ok) {
                Some(entry) => {
                    let path = entry.path();
                    if path.is_dir() {
                        if let Ok(rd) = fs::read_dir(&path) {
                            self.stack.push(rd);
                        }
                    } else if path.is_file() {
                        return Some(Value::Path(path));
                    }
                }
                None => {
                    self.stack.pop();
                }
            }
        }
        None
    }
}

struct Walk {
    target: Box<dyn Evaluator>,
}
impl Evaluator for Walk {
    fn expected_type(&self) -> ValueType {
        ValueType::List(Rc::new(ValueType::Path))
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::Path(path) = self.target.eval(file) else {
            return Value::Empty;
        };
        let walker = Walker::new(path);
        Value::List(List::new_lazy(Rc::new(ValueType::Path), walker))
    }
}

pub(super) fn new_walker(target: Box<dyn Evaluator>) -> Result<Box<dyn Evaluator>, FindItError> {
    match target.expected_type() {
        ValueType::Path => Ok(Box::new(Walk { target })),
        _ => Err(FindItError::BadExpression(
            "Walk method can only be applied to Path types".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::{
        errors::FindItError, evaluators::expr::read_expr, file_wrapper::FileWrapper, value::Value,
    };

    #[test]
    fn test_happy_path() -> Result<(), FindItError> {
        let expr = read_expr(
            "@\"tests/test_cases/order_by/test_files/next/emma\".walk().filter($f $f.extension == \"json\").map($f $f.name).sort() as text",
        )?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(
            expr.eval(file),
            Value::String(
                "[amelia-19.json, big-13.json, life-13.json, life-21.json, sophia-27.json]".into(),
            )
        );

        Ok(())
    }

    #[test]
    fn not_dir_empty_list() -> Result<(), FindItError> {
        let expr = read_expr("me.walk().length()")?;
        let path = Path::new("tests/test_cases/order_by/test_files/next/emma/amelia/big-13.json");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Number(0,));

        Ok(())
    }

    #[test]
    fn no_such_file_return_empty() -> Result<(), FindItError> {
        let expr = read_expr("(me.content as PATH).walk()")?;
        let path = Path::new("/no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty,);

        Ok(())
    }

    #[test]
    fn not_a_file_return_err() -> Result<(), FindItError> {
        let err = read_expr("12.walk()").err();

        assert!(err.is_some());

        Ok(())
    }
}
