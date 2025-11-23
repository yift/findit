use std::ops::Deref;

use crate::{
    errors::FindItError,
    evaluators::expr::{BindingsTypes, Evaluator, EvaluatorFactory},
    file_wrapper::FileWrapper,
    parser::ast::expression::Expression,
    value::{Value, ValueType},
};

struct IndexOf {
    target: Box<dyn Evaluator>,
    item_to_find: Box<dyn Evaluator>,
}
impl Evaluator for IndexOf {
    fn expected_type(&self) -> ValueType {
        ValueType::Number
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::List(target_value) = self.target.eval(file) else {
            return Value::Empty;
        };
        let item = self.item_to_find.eval(file);

        target_value
            .items()
            .into_iter()
            .enumerate()
            .filter(|f| f.1 == item)
            .map(|f| f.0)
            .map(|i| i.into())
            .next()
            .unwrap_or(Value::Empty)
    }
}
pub(super) fn new_index_of(
    target: Box<dyn Evaluator>,
    item_to_find: &Expression,
    bindings: &BindingsTypes,
) -> Result<Box<dyn Evaluator>, FindItError> {
    let ValueType::List(items_type) = target.expected_type() else {
        return Err(FindItError::BadExpression(
            "IndexOf method can only be applied to List type".to_string(),
        ));
    };

    let item_to_find = item_to_find.build(bindings)?;
    if &item_to_find.expected_type() != items_type.deref() {
        return Err(FindItError::BadExpression(
            "IndexOf item must be the same as the list items".to_string(),
        ));
    }
    Ok(Box::new(IndexOf {
        target,
        item_to_find,
    }))
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
    fn index_of_returns_the_index() -> Result<(), FindItError> {
        let expr = read_expr("[1, 2, 10, 4, 2, 5, 12].index_of(10)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Number(2));

        Ok(())
    }

    #[test]
    fn index_of_returns_the_first_index() -> Result<(), FindItError> {
        let expr = read_expr("[1, 2, 10, 4, 2, 5, 12].indexOf(2)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Number(1));

        Ok(())
    }

    #[test]
    fn index_of_return_empty_when_needed() -> Result<(), FindItError> {
        let expr = read_expr("[1, 2, 10, 4, 2, 5, 12].index_of(11)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn index_of_return_type() -> Result<(), FindItError> {
        let expr = read_expr("[1, 2, 4, 5].index_of(3)")?;

        assert_eq!(expr.expected_type(), ValueType::Number);

        Ok(())
    }

    #[test]
    fn index_of_returns_empty_when_needed() -> Result<(), FindItError> {
        let expr = read_expr("lines().index_of(\"one\")")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn index_of_no_list() {
        let err = read_expr("123.index_of(1)").err();
        assert!(err.is_some())
    }

    #[test]
    fn index_of_different_value_types() {
        let err = read_expr("[1, 2, 3].index_of(true)").err();
        assert!(err.is_some())
    }

    #[test]
    fn index_of_no_item() {
        let err = read_expr("[1, 2, 3].index_of()").err();
        assert!(err.is_some())
    }
}
