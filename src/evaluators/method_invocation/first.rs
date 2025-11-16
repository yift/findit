use std::ops::Deref;

use crate::{
    errors::FindItError,
    evaluators::expr::Evaluator,
    file_wrapper::FileWrapper,
    value::{Value, ValueType},
};

struct First {
    target: Box<dyn Evaluator>,
    item_type: ValueType,
}
impl Evaluator for First {
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
            .next()
            .unwrap_or(Value::Empty)
    }
}
pub(super) fn new_first(target: Box<dyn Evaluator>) -> Result<Box<dyn Evaluator>, FindItError> {
    match target.expected_type() {
        ValueType::List(item_type) => {
            let item_type = item_type.deref().clone();
            Ok(Box::new(First { target, item_type }))
        }
        _ => Err(FindItError::BadExpression(
            "First method can only be applied to List type".to_string(),
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
    fn test_first_empty() -> Result<(), FindItError> {
        let expr = read_expr(":[].first()")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn test_first_non_empty() -> Result<(), FindItError> {
        let expr = read_expr(":[1, 2, 4, 5].skip(2).first()")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Number(4));

        Ok(())
    }

    #[test]
    fn first_returns_empty_when_needed() -> Result<(), FindItError> {
        let expr = read_expr("lines().first()")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn first_return_type() -> Result<(), FindItError> {
        let expr = read_expr(":[1, 2, 4, 5].first()")?;

        assert_eq!(expr.expected_type(), ValueType::Number);

        Ok(())
    }

    #[test]
    fn first_no_list() {
        let err = read_expr("123.first()").err();
        assert!(err.is_some())
    }
}
