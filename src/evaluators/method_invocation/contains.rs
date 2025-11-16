use std::ops::Deref;

use itertools::Itertools;

use crate::{
    errors::FindItError,
    evaluators::expr::{BindingsTypes, Evaluator, EvaluatorFactory},
    file_wrapper::FileWrapper,
    parser::ast::expression::Expression,
    value::{Value, ValueType},
};

struct Contains {
    target: Box<dyn Evaluator>,
    item_to_find: Box<dyn Evaluator>,
}
impl Evaluator for Contains {
    fn expected_type(&self) -> ValueType {
        ValueType::Bool
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::List(target_value) = self.target.eval(file) else {
            return Value::Empty;
        };
        let item = self.item_to_find.eval(file);

        target_value.items().into_iter().contains(&item).into()
    }
}
pub(super) fn new_contains(
    target: Box<dyn Evaluator>,
    item_to_find: &Expression,
    bindings: &BindingsTypes,
) -> Result<Box<dyn Evaluator>, FindItError> {
    let ValueType::List(items_type) = target.expected_type() else {
        return Err(FindItError::BadExpression(
            "Contains method can only be applied to List type".to_string(),
        ));
    };

    let item_to_find = item_to_find.build(bindings)?;
    if &item_to_find.expected_type() != items_type.deref() {
        return Err(FindItError::BadExpression(
            "Contains item must be the same as the list items".to_string(),
        ));
    }
    Ok(Box::new(Contains {
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
    fn contains_returns_true_when_needed() -> Result<(), FindItError> {
        let expr = read_expr(":[1, 2, 10, 4, 2, 5, 12].contains(5)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Bool(true));

        Ok(())
    }

    #[test]
    fn contains_returns_false_when_needed() -> Result<(), FindItError> {
        let expr = read_expr(":[1, 2, 10, 4, 2, 5, 12].contains(11)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Bool(false));

        Ok(())
    }


    #[test]
    fn contains_returns_empty_when_needed() -> Result<(), FindItError> {
        let expr = read_expr("lines().contains(\"text\")")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn contains_return_type() -> Result<(), FindItError> {
        let expr = read_expr(":[1, 2, 4, 5].contains(3)")?;

        assert_eq!(expr.expected_type(), ValueType::Bool);

        Ok(())
    }

    #[test]
    fn contains_no_list() {
        let err = read_expr("123.contains(1)").err();
        assert!(err.is_some())
    }

    #[test]
    fn contains_different_value_types() {
        let err = read_expr(":[1, 2, 3].contains(true)").err();
        assert!(err.is_some())
    }

    #[test]
    fn contains_no_item() {
        let err = read_expr(":[1, 2, 3].contains()").err();
        assert!(err.is_some())
    }
}
