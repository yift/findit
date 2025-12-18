use crate::{
    errors::FindItError,
    evaluators::expr::Evaluator,
    file_wrapper::FileWrapper,
    value::{Value, ValueType},
};

struct Length {
    target: Box<dyn Evaluator>,
}
impl Evaluator for Length {
    fn expected_type(&self) -> ValueType {
        ValueType::Number
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let target_value = self.target.eval(file);
        match target_value {
            Value::List(list) => list.count().into(),
            Value::String(s) => s.len().into(),
            Value::Path(f) => {
                if let Ok(metadata) = std::fs::metadata(&f)
                    && metadata.is_file()
                    && let Ok(content) = std::fs::read(&f)
                {
                    content.len().into()
                } else {
                    Value::Empty
                }
            }
            _ => Value::Empty,
        }
    }
}

pub(super) fn new_length(target: Box<dyn Evaluator>) -> Result<Box<dyn Evaluator>, FindItError> {
    match target.expected_type() {
        ValueType::List(_) | ValueType::String | ValueType::Path => Ok(Box::new(Length { target })),
        _ => Err(FindItError::BadExpression(
            "Length method can only be applied to List, String or Path types".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use crate::{
        errors::FindItError,
        evaluators::expr::read_expr,
        file_wrapper::FileWrapper,
        value::{Value, ValueType},
    };

    #[test]
    fn test_simple_len() -> Result<(), FindItError> {
        let expr = read_expr("[1, 2, 3].len()")?;
        let file = &FileWrapper::new(PathBuf::new(), 1);

        assert_eq!(expr.eval(file), Value::Number(3));

        Ok(())
    }

    #[test]
    fn length_no_string_expr() {
        let err = read_expr("12.LEN()").err();
        assert!(err.is_some())
    }

    #[test]
    fn length_null_str_return_empty() {
        let eval = read_expr("content.len()").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn length_expect_number() {
        let expr = read_expr("\"test\".len()").unwrap();
        assert_eq!(expr.expected_type(), ValueType::Number);
    }

    #[test]
    fn length_return_the_correct_value() {
        let eval = read_expr("\"123\".len()").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Number(3))
    }

    #[test]
    fn length_as_property() {
        let eval = read_expr("\"abcd\".len").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Number(4))
    }

    #[test]
    fn test_length_expected_type() -> Result<(), FindItError> {
        test_expected_type("length()", ValueType::Number)
    }

    fn test_expected_type(name: &str, expected: ValueType) -> Result<(), FindItError> {
        let expr = read_expr(name)?;
        let tp = expr.expected_type();

        assert_eq!(tp, expected);

        Ok(())
    }
}
