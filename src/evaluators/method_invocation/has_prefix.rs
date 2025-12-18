use crate::{
    errors::FindItError,
    evaluators::expr::{BindingsTypes, Evaluator, EvaluatorFactory},
    file_wrapper::FileWrapper,
    parser::ast::expression::Expression,
    value::{Value, ValueType},
};

struct HasPrefix {
    target: Box<dyn Evaluator>,
    prefix: Box<dyn Evaluator>,
}
impl Evaluator for HasPrefix {
    fn expected_type(&self) -> ValueType {
        ValueType::Bool
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::String(target_value) = self.target.eval(file) else {
            return Value::Empty;
        };
        let Value::String(prefix) = self.prefix.eval(file) else {
            return Value::Empty;
        };
        target_value.starts_with(&prefix).into()
    }
}
pub(super) fn new_has_prefix(
    target: Box<dyn Evaluator>,
    prefix: &Expression,
    bindings: &BindingsTypes,
) -> Result<Box<dyn Evaluator>, FindItError> {
    if target.expected_type() != ValueType::String {
        return Err(FindItError::BadExpression(
            "HasPrefix method can only be applied to String type".to_string(),
        ));
    }
    let prefix = prefix.build(bindings)?;
    if prefix.expected_type() != ValueType::String {
        return Err(FindItError::BadExpression(
            "HasPrefix method prefix must be a String".to_string(),
        ));
    }
    Ok(Box::new(HasPrefix { target, prefix }))
}
#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::{
        errors::FindItError,
        evaluators::expr::read_expr,
        file_wrapper::FileWrapper,
        value::{Value, ValueType},
    };

    #[test]
    fn test_has_prefix_true() -> Result<(), FindItError> {
        let expr = read_expr("\"abc\".has_prefix(\"a\")")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), true.into());

        Ok(())
    }

    #[test]
    fn test_has_prefix_false() -> Result<(), FindItError> {
        let expr = read_expr("\"abc\".has_prefix(\"b\")")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), false.into());

        Ok(())
    }

    #[test]
    fn test_has_prefix_no_target() -> Result<(), FindItError> {
        let expr = read_expr("content.has_prefix(\"a\")")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn test_has_prefix_empty_prefix() -> Result<(), FindItError> {
        let expr = read_expr("\"abc\".has_prefix(content)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn test_has_prefix_return_type() -> Result<(), FindItError> {
        let expr = read_expr("\"abc\".has_prefix(\"a\")")?;

        assert_eq!(expr.expected_type(), ValueType::Bool);

        Ok(())
    }

    #[test]
    fn test_has_prefix_no_str() -> Result<(), FindItError> {
        let expr = read_expr("\"abc\".has_prefix(12)").err();

        assert!(expr.is_some());

        Ok(())
    }

    #[test]
    fn test_has_prefix_no_str_two() -> Result<(), FindItError> {
        let expr = read_expr("12.has_prefix(\"a\")").err();

        assert!(expr.is_some());

        Ok(())
    }
}
