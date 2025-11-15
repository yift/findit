use std::{
    fs::File,
    io::{BufRead, BufReader},
    rc::Rc,
};

use crate::{
    errors::FindItError,
    evaluators::expr::Evaluator,
    file_wrapper::FileWrapper,
    value::{List, Value, ValueType},
};

struct LinesString {
    target: Box<dyn Evaluator>,
}
impl Evaluator for LinesString {
    fn expected_type(&self) -> ValueType {
        ValueType::List(Rc::new(ValueType::String))
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::String(str) = self.target.eval(file) else {
            return Value::Empty;
        };
        let items = str.lines().map(|s| Value::String(s.to_string()));
        Value::List(List::new_eager(Rc::new(ValueType::String), items))
    }
}

struct LinesFile {
    target: Box<dyn Evaluator>,
}
impl Evaluator for LinesFile {
    fn expected_type(&self) -> ValueType {
        ValueType::List(Rc::new(ValueType::String))
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::Path(path) = self.target.eval(file) else {
            return Value::Empty;
        };
        let Ok(file) = File::open(path) else {
            return Value::Empty;
        };
        let buf = BufReader::new(file);
        let items = buf.lines().map_while(Result::ok).map(Value::String);
        Value::List(List::new_lazy(Rc::new(ValueType::String), items))
    }
}

pub(super) fn new_lines(target: Box<dyn Evaluator>) -> Result<Box<dyn Evaluator>, FindItError> {
    match target.expected_type() {
        ValueType::String => Ok(Box::new(LinesString { target })),
        ValueType::Path => Ok(Box::new(LinesFile { target })),
        _ => Err(FindItError::BadExpression(
            "Lines method can only be applied to String or Path types".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use std::{path::Path, rc::Rc};

    use crate::{
        errors::FindItError,
        evaluators::expr::read_expr,
        file_wrapper::FileWrapper,
        value::{List, Value, ValueType},
    };

    #[test]
    fn test_lines_string() -> Result<(), FindItError> {
        let expr = read_expr("\"one\ntwo\nthree\n\".lines()")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(
            expr.eval(file),
            Value::List(List::new_eager(
                Rc::new(ValueType::String),
                vec![
                    Value::String("one".into()),
                    Value::String("two".into()),
                    Value::String("three".into())
                ]
                .into_iter(),
            ))
        );

        Ok(())
    }

    #[test]
    fn test_lines_string_no_target() -> Result<(), FindItError> {
        let expr = read_expr("content.lines()")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn test_lines_string_return_type() -> Result<(), FindItError> {
        let expr = read_expr("\"abc\".lines()")?;

        assert_eq!(
            expr.expected_type(),
            ValueType::List(Rc::new(ValueType::String))
        );

        Ok(())
    }
    #[test]
    fn test_lines_number() {
        let expr = read_expr("12.lines()").err();

        assert!(expr.is_some());
    }

    #[test]
    fn test_lines_file() -> Result<(), FindItError> {
        let expr = read_expr("lines()")?;
        let path = Path::new("tests/test_cases/display/test_files/week-362.txt");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(
            expr.eval(file),
            Value::List(List::new_eager(
                Rc::new(ValueType::String),
                vec![
                    Value::String("quo eligendi amet harum ullam minus quasi ut.".into()),
                    Value::String("magni neque sed est incidunt expedita.".into()),
                    Value::String(
                        "quia quasi illo perferendis doloremque illum qui voluptas ullam.".into()
                    ),
                    Value::String("ab nulla nobis maiores nobis beatae velit ea quia.".into()),
                    Value::String("adipisci debitis facilis molestiae soluta repellat aut.".into()),
                    Value::String("vero libero repudiandae fugiat ducimus occaecati.".into()),
                    Value::String("".into()),
                ]
                .into_iter(),
            ))
        );

        Ok(())
    }

    #[test]
    fn test_lines_file_return_type() -> Result<(), FindItError> {
        let expr = read_expr("lines()")?;

        assert_eq!(
            expr.expected_type(),
            ValueType::List(Rc::new(ValueType::String))
        );

        Ok(())
    }

    #[test]
    fn test_lines_file_no_target() -> Result<(), FindItError> {
        let expr = read_expr("parent.lines()")?;
        let path = Path::new("/");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty,);

        Ok(())
    }

    #[test]
    fn test_lines_file_target_not_a_file() -> Result<(), FindItError> {
        let expr = read_expr("parent.lines()")?;
        let path = Path::new(".");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty,);

        Ok(())
    }
}
