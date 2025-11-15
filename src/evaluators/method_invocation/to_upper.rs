use crate::{
    errors::FindItError,
    evaluators::expr::Evaluator,
    file_wrapper::FileWrapper,
    value::{Value, ValueType},
};

struct ToUpper {
    target: Box<dyn Evaluator>,
}

impl Evaluator for ToUpper {
    fn expected_type(&self) -> ValueType {
        ValueType::String
    }

    fn eval(&self, file: &FileWrapper) -> Value {
        let target_value = self.target.eval(file);
        match target_value {
            Value::String(s) => s.to_uppercase().into(),
            _ => Value::Empty,
        }
    }
}

pub(super) fn new_to_upper(target: Box<dyn Evaluator>) -> Result<Box<dyn Evaluator>, FindItError> {
    match target.expected_type() {
        ValueType::String => Ok(Box::new(ToUpper { target })),
        _ => Err(FindItError::BadExpression(
            "ToUpper method can only be applied to String type".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::{
        evaluators::expr::read_expr,
        file_wrapper::FileWrapper,
        value::{Value, ValueType},
    };

    #[test]
    fn upper_no_string_expr() {
        let err = read_expr("12.Upper()").err();
        assert!(err.is_some())
    }

    #[test]
    fn upper_no_args() {
        let err = read_expr("upper()").err();
        assert!(err.is_some())
    }

    #[test]
    fn upper_too_many_args() {
        let err = read_expr("\"abc\".upper(\"def\")").err();
        assert!(err.is_some())
    }

    #[test]
    fn upper_null_str_return_empty() {
        let eval = read_expr("content.upper()").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn upper_expect_string() {
        let expr = read_expr("\"test\".upper()").unwrap();
        assert_eq!(expr.expected_type(), ValueType::String);
    }

    #[test]
    fn upper_return_the_correct_value() {
        let eval = read_expr("\"abcDEF\".upper()").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::String("ABCDEF".into()))
    }
}
