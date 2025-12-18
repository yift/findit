use crate::{
    errors::FindItError,
    evaluators::expr::{BindingsTypes, Evaluator, EvaluatorFactory},
    file_wrapper::FileWrapper,
    parser::ast::expression::Expression,
    value::{Value, ValueType},
};

struct RemoveSuffix {
    target: Box<dyn Evaluator>,
    suffix: Box<dyn Evaluator>,
}
impl Evaluator for RemoveSuffix {
    fn expected_type(&self) -> ValueType {
        ValueType::String
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::String(target_value) = self.target.eval(file) else {
            return Value::Empty;
        };
        let Value::String(suffix) = self.suffix.eval(file) else {
            return Value::Empty;
        };
        if target_value.ends_with(&suffix) {
            target_value[..target_value.len() - suffix.len()].into()
        } else {
            Value::String(target_value)
        }
    }
}
pub(super) fn new_remove_suffix(
    target: Box<dyn Evaluator>,
    suffix: &Expression,
    bindings: &BindingsTypes,
) -> Result<Box<dyn Evaluator>, FindItError> {
    if target.expected_type() != ValueType::String {
        return Err(FindItError::BadExpression(
            "RemoveSuffix method can only be applied to String type".to_string(),
        ));
    }
    let suffix = suffix.build(bindings)?;
    if suffix.expected_type() != ValueType::String {
        return Err(FindItError::BadExpression(
            "RemoveSuffix method suffix must be a String".to_string(),
        ));
    }
    Ok(Box::new(RemoveSuffix { target, suffix }))
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
    fn test_remove_suffix_with_value() -> Result<(), FindItError> {
        let expr = read_expr("\"abc\".remove_suffix(\"bc\")")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), "a".into());

        Ok(())
    }

    #[test]
    fn test_remove_suffix_without_value() -> Result<(), FindItError> {
        let expr = read_expr("\"abc\".remove_suffix(\"b\")")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), "abc".into());
        Ok(())
    }

    #[test]
    fn test_remove_suffix_no_target() -> Result<(), FindItError> {
        let expr = read_expr("content.remove_suffix(\"a\")")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn test_remove_suffix_empty_suffix() -> Result<(), FindItError> {
        let expr = read_expr("\"abc\".remove_suffix(content)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn test_remove_suffix_return_type() -> Result<(), FindItError> {
        let expr = read_expr("\"abc\".remove_suffix(\"a\")")?;

        assert_eq!(expr.expected_type(), ValueType::String);

        Ok(())
    }

    #[test]
    fn test_remove_suffix_no_str() -> Result<(), FindItError> {
        let expr = read_expr("\"abc\".remove_suffix(12)").err();

        assert!(expr.is_some());

        Ok(())
    }

    #[test]
    fn test_remove_suffix_no_str_two() -> Result<(), FindItError> {
        let expr = read_expr("12.remove_suffix(\"a\")").err();

        assert!(expr.is_some());

        Ok(())
    }
}
