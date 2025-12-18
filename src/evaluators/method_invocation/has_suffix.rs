use crate::{
    errors::FindItError,
    evaluators::expr::{BindingsTypes, Evaluator, EvaluatorFactory},
    file_wrapper::FileWrapper,
    parser::ast::expression::Expression,
    value::{Value, ValueType},
};

struct HasSuffix {
    target: Box<dyn Evaluator>,
    suffix: Box<dyn Evaluator>,
}
impl Evaluator for HasSuffix {
    fn expected_type(&self) -> ValueType {
        ValueType::Bool
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::String(target_value) = self.target.eval(file) else {
            return Value::Empty;
        };
        let Value::String(suffix) = self.suffix.eval(file) else {
            return Value::Empty;
        };
        target_value.ends_with(&suffix).into()
    }
}
pub(super) fn new_has_suffix(
    target: Box<dyn Evaluator>,
    suffix: &Expression,
    bindings: &BindingsTypes,
) -> Result<Box<dyn Evaluator>, FindItError> {
    if target.expected_type() != ValueType::String {
        return Err(FindItError::BadExpression(
            "HasSuffix method can only be applied to String type".to_string(),
        ));
    }
    let suffix = suffix.build(bindings)?;
    if suffix.expected_type() != ValueType::String {
        return Err(FindItError::BadExpression(
            "HasSuffix method suffix must be a String".to_string(),
        ));
    }
    Ok(Box::new(HasSuffix { target, suffix }))
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
    fn test_has_suffix_true() -> Result<(), FindItError> {
        let expr = read_expr("\"abc\".has_suffix(\"c\")")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), true.into());

        Ok(())
    }

    #[test]
    fn test_has_suffix_false() -> Result<(), FindItError> {
        let expr = read_expr("\"abc\".has_suffix(\"b\")")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), false.into());

        Ok(())
    }

    #[test]
    fn test_has_suffix_no_target() -> Result<(), FindItError> {
        let expr = read_expr("content.has_suffix(\"a\")")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn test_has_suffix_empty_suffix() -> Result<(), FindItError> {
        let expr = read_expr("\"abc\".has_suffix(content)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn test_has_suffix_return_type() -> Result<(), FindItError> {
        let expr = read_expr("\"abc\".has_suffix(\"a\")")?;

        assert_eq!(expr.expected_type(), ValueType::Bool);

        Ok(())
    }

    #[test]
    fn test_has_suffix_no_str() -> Result<(), FindItError> {
        let expr = read_expr("\"abc\".has_suffix(12)").err();

        assert!(expr.is_some());

        Ok(())
    }

    #[test]
    fn test_has_suffix_no_str_two() -> Result<(), FindItError> {
        let expr = read_expr("12.has_suffix(\"a\")").err();

        assert!(expr.is_some());

        Ok(())
    }
}
