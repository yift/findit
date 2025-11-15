use std::rc::Rc;

use crate::{
    errors::FindItError,
    evaluators::expr::{BindingsTypes, Evaluator, EvaluatorFactory},
    file_wrapper::FileWrapper,
    parser::ast::expression::Expression,
    value::{List, Value, ValueType},
};

struct Split {
    target: Box<dyn Evaluator>,
    delimiter: Box<dyn Evaluator>,
}
impl Evaluator for Split {
    fn expected_type(&self) -> ValueType {
        ValueType::List(Rc::new(ValueType::String))
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::String(target_value) = self.target.eval(file) else {
            return Value::Empty;
        };
        let Value::String(delimiter) = self.delimiter.eval(file) else {
            return Value::Empty;
        };
        if delimiter.is_empty() {
            return Value::Empty;
        }
        let items = target_value
            .split(&delimiter)
            .map(|s| Value::String(s.to_string()));
        Value::List(List::new_eager(Rc::new(ValueType::String), items))
    }
}
pub(super) fn new_split(
    target: Box<dyn Evaluator>,
    delimiter: &Expression,
    bindings: &BindingsTypes,
) -> Result<Box<dyn Evaluator>, FindItError> {
    if target.expected_type() != ValueType::String {
        return Err(FindItError::BadExpression(
            "Split method can only be applied to String type".to_string(),
        ));
    }
    let delimiter = delimiter.build(bindings)?;
    if delimiter.expected_type() != ValueType::String {
        return Err(FindItError::BadExpression(
            "Split method delimiter must be a String".to_string(),
        ));
    }
    Ok(Box::new(Split { target, delimiter }))
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
    fn test_split() -> Result<(), FindItError> {
        let expr = read_expr("\"a|b|c\".split(\"|\")")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(
            expr.eval(file),
            Value::List(List::new_eager(
                Rc::new(ValueType::String),
                vec![
                    Value::String("a".into()),
                    Value::String("b".into()),
                    Value::String("c".into())
                ]
                .into_iter(),
            ))
        );

        Ok(())
    }

    #[test]
    fn test_split_no_delim() -> Result<(), FindItError> {
        let expr = read_expr("\"a|b|c\".split(\"\")")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn test_split_no_target() -> Result<(), FindItError> {
        let expr = read_expr("content.split(\"|\")")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn test_split_empty_delim() -> Result<(), FindItError> {
        let expr = read_expr("\"a|b|c\".split(content)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn test_split_return_type() -> Result<(), FindItError> {
        let expr = read_expr("\"a|b|c\".split(\"\")")?;

        assert_eq!(
            expr.expected_type(),
            ValueType::List(Rc::new(ValueType::String))
        );

        Ok(())
    }

    #[test]
    fn test_split_no_str() {
        let expr = read_expr("\"a|b|c\".split(12)").err();

        assert!(expr.is_some());
    }

    #[test]
    fn test_split_no_str_two() {
        let expr = read_expr("12.split(\"a\")").err();

        assert!(expr.is_some());
    }
}
