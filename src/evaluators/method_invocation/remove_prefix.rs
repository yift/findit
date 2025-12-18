use crate::{
    errors::FindItError,
    evaluators::expr::{BindingsTypes, Evaluator, EvaluatorFactory},
    file_wrapper::FileWrapper,
    parser::ast::expression::Expression,
    value::{Value, ValueType},
};

struct RemovePrefix {
    target: Box<dyn Evaluator>,
    prefix: Box<dyn Evaluator>,
}
impl Evaluator for RemovePrefix {
    fn expected_type(&self) -> ValueType {
        ValueType::String
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::String(target_value) = self.target.eval(file) else {
            return Value::Empty;
        };
        let Value::String(prefix) = self.prefix.eval(file) else {
            return Value::Empty;
        };
        if target_value.starts_with(&prefix) {
            target_value[prefix.len()..].into()
        } else {
            Value::String(target_value)
        }
    }
}
pub(super) fn new_remove_prefix(
    target: Box<dyn Evaluator>,
    prefix: &Expression,
    bindings: &BindingsTypes,
) -> Result<Box<dyn Evaluator>, FindItError> {
    if target.expected_type() != ValueType::String {
        return Err(FindItError::BadExpression(
            "RemovePrefix method can only be applied to String type".to_string(),
        ));
    }
    let prefix = prefix.build(bindings)?;
    if prefix.expected_type() != ValueType::String {
        return Err(FindItError::BadExpression(
            "RemovePrefix method prefix must be a String".to_string(),
        ));
    }
    Ok(Box::new(RemovePrefix { target, prefix }))
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
    fn test_remove_prefix_with_value() -> Result<(), FindItError> {
        let expr = read_expr("\"abc\".remove_prefix(\"ab\")")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), "c".into());

        Ok(())
    }

    #[test]
    fn test_remove_prefix_without_value() -> Result<(), FindItError> {
        let expr = read_expr("\"abc\".remove_prefix(\"b\")")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), "abc".into());
        Ok(())
    }

    #[test]
    fn test_remove_prefix_no_target() -> Result<(), FindItError> {
        let expr = read_expr("content.remove_prefix(\"a\")")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn test_remove_prefix_empty_prefix() -> Result<(), FindItError> {
        let expr = read_expr("\"abc\".remove_prefix(content)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn test_remove_prefix_return_type() -> Result<(), FindItError> {
        let expr = read_expr("\"abc\".remove_prefix(\"a\")")?;

        assert_eq!(expr.expected_type(), ValueType::String);

        Ok(())
    }

    #[test]
    fn test_remove_prefix_no_str() -> Result<(), FindItError> {
        let expr = read_expr("\"abc\".remove_prefix(12)").err();

        assert!(expr.is_some());

        Ok(())
    }

    #[test]
    fn test_remove_prefix_no_str_two() -> Result<(), FindItError> {
        let expr = read_expr("12.remove_prefix(\"a\")").err();

        assert!(expr.is_some());

        Ok(())
    }
}
