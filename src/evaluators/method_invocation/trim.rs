use crate::{
    errors::FindItError,
    evaluators::expr::Evaluator,
    file_wrapper::FileWrapper,
    value::{Value, ValueType},
};

struct Trim {
    target: Box<dyn Evaluator>,
}
impl Evaluator for Trim {
    fn expected_type(&self) -> ValueType {
        ValueType::String
    }

    fn eval(&self, file: &FileWrapper) -> Value {
        let target_value = self.target.eval(file);
        match target_value {
            Value::String(s) => s.trim().into(),
            _ => Value::Empty,
        }
    }
}

struct TrimHead {
    target: Box<dyn Evaluator>,
}
impl Evaluator for TrimHead {
    fn expected_type(&self) -> ValueType {
        ValueType::String
    }

    fn eval(&self, file: &FileWrapper) -> Value {
        let target_value = self.target.eval(file);
        match target_value {
            Value::String(s) => s.trim_start().into(),
            _ => Value::Empty,
        }
    }
}

struct TrimTail {
    target: Box<dyn Evaluator>,
}
impl Evaluator for TrimTail {
    fn expected_type(&self) -> ValueType {
        ValueType::String
    }

    fn eval(&self, file: &FileWrapper) -> Value {
        let target_value = self.target.eval(file);
        match target_value {
            Value::String(s) => s.trim_end().into(),
            _ => Value::Empty,
        }
    }
}

pub(super) fn new_trim(target: Box<dyn Evaluator>) -> Result<Box<dyn Evaluator>, FindItError> {
    match target.expected_type() {
        ValueType::String => Ok(Box::new(Trim { target })),
        _ => Err(FindItError::BadExpression(
            "Trim method can only be applied to String type".to_string(),
        )),
    }
}
pub(super) fn new_trim_head(target: Box<dyn Evaluator>) -> Result<Box<dyn Evaluator>, FindItError> {
    match target.expected_type() {
        ValueType::String => Ok(Box::new(TrimHead { target })),
        _ => Err(FindItError::BadExpression(
            "TrimHead method can only be applied to String type".to_string(),
        )),
    }
}
pub(super) fn new_trim_tail(target: Box<dyn Evaluator>) -> Result<Box<dyn Evaluator>, FindItError> {
    match target.expected_type() {
        ValueType::String => Ok(Box::new(TrimTail { target })),
        _ => Err(FindItError::BadExpression(
            "TrimTail method can only be applied to String type".to_string(),
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
    fn trim_no_string_expr() {
        let err = read_expr("12.TRIM()").err();
        assert!(err.is_some())
    }

    #[test]
    fn trim_no_args() {
        let err = read_expr("TRIM()").err();
        assert!(err.is_some())
    }

    #[test]
    fn trim_too_many_args() {
        let err = read_expr("\"abc\".TRIM(\"def\")").err();
        assert!(err.is_some())
    }

    #[test]
    fn trim_null_str_return_empty() {
        let eval = read_expr("content.TRIM()").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn trim_expect_string() {
        let expr = read_expr("\"\".TRIM()").unwrap();
        assert_eq!(expr.expected_type(), ValueType::String);
    }

    #[test]
    fn trim_head_null_str_return_empty() {
        let eval = read_expr("content.TRIM_head()").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn trim_head_expect_string() {
        let expr = read_expr("\"\".trim_head()").unwrap();
        assert_eq!(expr.expected_type(), ValueType::String);
    }

    #[test]
    fn trim_tail_null_str_return_empty() {
        let eval = read_expr("content.TRIM_tail()").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn trim_tail_expect_string() {
        let expr = read_expr("\"\".TRIM_tail()").unwrap();
        assert_eq!(expr.expected_type(), ValueType::String);
    }

    #[test]
    fn length_no_string_trim_head() {
        let err = read_expr("12.trimHead()").err();
        assert!(err.is_some())
    }

    #[test]
    fn length_no_string_trim_tail() {
        let err = read_expr("12.trimTail()").err();
        assert!(err.is_some())
    }
}
