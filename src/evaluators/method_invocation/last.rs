use std::ops::Deref;

use crate::{
    errors::FindItError,
    evaluators::expr::Evaluator,
    file_wrapper::FileWrapper,
    value::{Value, ValueType},
};

struct Last {
    target: Box<dyn Evaluator>,
    item_type: ValueType,
}
impl Evaluator for Last {
    fn expected_type(&self) -> ValueType {
        self.item_type.clone()
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::List(target_value) = self.target.eval(file) else {
            return Value::Empty;
        };
        target_value
            .items()
            .into_iter()
            .last()
            .unwrap_or(Value::Empty)
    }
}
pub(super) fn new_last(target: Box<dyn Evaluator>) -> Result<Box<dyn Evaluator>, FindItError> {
    match target.expected_type() {
        ValueType::List(item_type) => {
            let item_type = item_type.deref().clone();
            Ok(Box::new(Last { target, item_type }))
        }
        _ => Err(FindItError::BadExpression(
            "Last method can only be applied to List type".to_string(),
        )),
    }
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
    fn test_last_empty() -> Result<(), FindItError> {
        let expr = read_expr(":[].last()")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn test_last_non_empty() -> Result<(), FindItError> {
        let expr = read_expr(":[1, 2, 4, 5].take(3).last()")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Number(4));

        Ok(())
    }

    #[test]
    fn last_of_returns_empty_when_needed() -> Result<(), FindItError> {
        let expr = read_expr("lines().last()")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn last_return_type() -> Result<(), FindItError> {
        let expr = read_expr(":[1, 2, 4, 5].last()")?;

        assert_eq!(expr.expected_type(), ValueType::Number);

        Ok(())
    }

    #[test]
    fn last_no_list() {
        let err = read_expr("123.last()").err();
        assert!(err.is_some())
    }
}
