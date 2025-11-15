use crate::{
    errors::FindItError,
    evaluators::expr::Evaluator,
    file_wrapper::FileWrapper,
    value::{Value, ValueType},
};

struct ToLower {
    target: Box<dyn Evaluator>,
}

impl Evaluator for ToLower {
    fn expected_type(&self) -> ValueType {
        ValueType::String
    }

    fn eval(&self, file: &FileWrapper) -> Value {
        let target_value = self.target.eval(file);
        match target_value {
            Value::String(s) => s.to_lowercase().into(),
            _ => Value::Empty,
        }
    }
}

pub(super) fn new_to_lower(target: Box<dyn Evaluator>) -> Result<Box<dyn Evaluator>, FindItError> {
    match target.expected_type() {
        ValueType::String => Ok(Box::new(ToLower { target })),
        _ => Err(FindItError::BadExpression(
            "ToLower method can only be applied to String type".to_string(),
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
    fn lower_no_string_expr() {
        let err = read_expr("12.Lower()").err();
        assert!(err.is_some())
    }

    #[test]
    fn lower_no_args() {
        let err = read_expr("lower()").err();
        assert!(err.is_some())
    }

    #[test]
    fn lower_too_many_args() {
        let err = read_expr("\"abc\".lower(\"def\")").err();
        assert!(err.is_some())
    }

    #[test]
    fn lower_null_str_return_empty() {
        let eval = read_expr("content.lower()").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn lower_expect_string() {
        let expr = read_expr("\"test\".lower()").unwrap();
        assert_eq!(expr.expected_type(), ValueType::String);
    }

    #[test]
    fn lower_return_the_correct_value() {
        let eval = read_expr("\"abcDEF\".lower()").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::String("abcdef".into()))
    }
}
