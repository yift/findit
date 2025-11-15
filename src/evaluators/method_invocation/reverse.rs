use crate::{
    errors::FindItError,
    evaluators::expr::Evaluator,
    file_wrapper::FileWrapper,
    value::{Value, ValueType},
};

struct Reverse {
    target: Box<dyn Evaluator>,
}
impl Evaluator for Reverse {
    fn expected_type(&self) -> ValueType {
        ValueType::String
    }

    fn eval(&self, file: &FileWrapper) -> Value {
        let target_value = self.target.eval(file);
        match target_value {
            Value::String(s) => s.chars().rev().collect::<String>().into(),
            _ => Value::Empty,
        }
    }
}

pub(super) fn new_reverse(target: Box<dyn Evaluator>) -> Result<Box<dyn Evaluator>, FindItError> {
    match target.expected_type() {
        ValueType::String => Ok(Box::new(Reverse { target })),
        _ => Err(FindItError::BadExpression(
            "Reverse method can only be applied to String type".to_string(),
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
    fn reverse_null_str_return_empty() {
        let eval = read_expr("content.REVERSE()").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn reverse_expect_string() {
        let expr = read_expr("\"\".REVERSE()").unwrap();
        assert_eq!(expr.expected_type(), ValueType::String);
    }

    #[test]
    fn reverse_works() {
        let eval = read_expr("\"123\".REVERSE()").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::String("321".into()))
    }

    #[test]
    fn length_no_string_reverse() {
        let err = read_expr("12.reverse()").err();
        assert!(err.is_some())
    }
}
