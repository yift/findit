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

struct StringWords {
    target: Box<dyn Evaluator>,
}
impl Evaluator for StringWords {
    fn expected_type(&self) -> ValueType {
        ValueType::List(Rc::new(ValueType::String))
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::String(target_value) = self.target.eval(file) else {
            return Value::Empty;
        };
        let items = target_value
            .split_whitespace()
            .map(|s| Value::String(s.to_string()));
        Value::List(List::new_eager(Rc::new(ValueType::String), items))
    }
}
struct FileWords {
    target: Box<dyn Evaluator>,
}
impl Evaluator for FileWords {
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
        let items = buf.lines().map_while(Result::ok).flat_map(|s| {
            s.split_whitespace()
                .map(|s| Value::String(s.into()))
                .collect::<Vec<_>>()
        });

        Value::List(List::new_lazy(Rc::new(ValueType::String), items))
    }
}

pub(super) fn new_words(target: Box<dyn Evaluator>) -> Result<Box<dyn Evaluator>, FindItError> {
    match target.expected_type() {
        ValueType::String => Ok(Box::new(StringWords { target })),
        ValueType::Path => Ok(Box::new(FileWords { target })),
        _ => Err(FindItError::BadExpression(
            "Words method can only be applied to String or Path types".to_string(),
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
    fn test_words_string() -> Result<(), FindItError> {
        let expr = read_expr("\"  one\ntwo  three \n    \".words()")?;
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
    fn test_words_string_no_target() -> Result<(), FindItError> {
        let expr = read_expr("content.words()")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn test_words_string_return_type() -> Result<(), FindItError> {
        let expr = read_expr("\"abc\".words()")?;

        assert_eq!(
            expr.expected_type(),
            ValueType::List(Rc::new(ValueType::String))
        );

        Ok(())
    }

    #[test]
    fn test_words_number() {
        let expr = read_expr("12.words()").err();

        assert!(expr.is_some());
    }

    #[test]
    fn test_words_file() -> Result<(), FindItError> {
        let expr = read_expr("words()")?;
        let path = Path::new("tests/test_cases/display/test_files/week-362.txt");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(
            expr.eval(file),
            Value::List(List::new_eager(
                Rc::new(ValueType::String),
                vec![
                    Value::String("quo".into()),
                    Value::String("eligendi".into()),
                    Value::String("amet".into()),
                    Value::String("harum".into()),
                    Value::String("ullam".into()),
                    Value::String("minus".into()),
                    Value::String("quasi".into()),
                    Value::String("ut.".into()),
                    Value::String("magni".into()),
                    Value::String("neque".into()),
                    Value::String("sed".into()),
                    Value::String("est".into()),
                    Value::String("incidunt".into()),
                    Value::String("expedita.".into()),
                    Value::String("quia".into()),
                    Value::String("quasi".into()),
                    Value::String("illo".into()),
                    Value::String("perferendis".into()),
                    Value::String("doloremque".into()),
                    Value::String("illum".into()),
                    Value::String("qui".into()),
                    Value::String("voluptas".into()),
                    Value::String("ullam.".into()),
                    Value::String("ab".into()),
                    Value::String("nulla".into()),
                    Value::String("nobis".into()),
                    Value::String("maiores".into()),
                    Value::String("nobis".into()),
                    Value::String("beatae".into()),
                    Value::String("velit".into()),
                    Value::String("ea".into()),
                    Value::String("quia.".into()),
                    Value::String("adipisci".into()),
                    Value::String("debitis".into()),
                    Value::String("facilis".into()),
                    Value::String("molestiae".into()),
                    Value::String("soluta".into()),
                    Value::String("repellat".into()),
                    Value::String("aut.".into()),
                    Value::String("vero".into()),
                    Value::String("libero".into()),
                    Value::String("repudiandae".into()),
                    Value::String("fugiat".into()),
                    Value::String("ducimus".into()),
                    Value::String("occaecati.".into()),
                ]
                .into_iter(),
            ))
        );

        Ok(())
    }

    #[test]
    fn test_words_file_no_target() -> Result<(), FindItError> {
        let expr = read_expr("parent.words()")?;
        let path = Path::new("/");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty,);

        Ok(())
    }

    #[test]
    fn test_words_file_target_not_a_file() -> Result<(), FindItError> {
        let expr = read_expr("parent.words()")?;
        let path = Path::new(".");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty,);

        Ok(())
    }

    #[test]
    fn test_words_file_return_type() -> Result<(), FindItError> {
        let expr = read_expr("words()")?;

        assert_eq!(
            expr.expected_type(),
            ValueType::List(Rc::new(ValueType::String))
        );

        Ok(())
    }
}
