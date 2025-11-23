use std::rc::Rc;

use crate::{
    errors::FindItError,
    evaluators::expr::{BindingsTypes, Evaluator, EvaluatorFactory},
    file_wrapper::FileWrapper,
    parser::ast::expression::Expression,
    value::{List, Value, ValueType},
};

struct TakeString {
    target: Box<dyn Evaluator>,
    limit: Box<dyn Evaluator>,
}
impl Evaluator for TakeString {
    fn expected_type(&self) -> ValueType {
        ValueType::String
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::String(target_value) = self.target.eval(file) else {
            return Value::Empty;
        };
        let Value::Number(limit_value) = self.limit.eval(file) else {
            return Value::Empty;
        };
        target_value
            .chars()
            .take(limit_value as usize)
            .collect::<String>()
            .into()
    }
}

struct TakeList {
    target: Box<dyn Evaluator>,
    limit: Box<dyn Evaluator>,
    items_type: Rc<ValueType>,
}
impl Evaluator for TakeList {
    fn expected_type(&self) -> ValueType {
        ValueType::List(self.items_type.clone())
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::List(target_value) = self.target.eval(file) else {
            return Value::Empty;
        };
        let Value::Number(limit) = self.limit.eval(file) else {
            return Value::Empty;
        };
        let iter = target_value.items().into_iter().take(limit as usize);
        Value::List(List::new_lazy(self.items_type.clone(), iter))
    }
}
pub(super) fn new_take(
    target: Box<dyn Evaluator>,
    limit: &Expression,
    bindings: &BindingsTypes,
) -> Result<Box<dyn Evaluator>, FindItError> {
    let limit = limit.build(bindings)?;

    if limit.expected_type() != ValueType::Number {
        return Err(FindItError::BadExpression(
            "Take method argument must be a Number".to_string(),
        ));
    }
    match target.expected_type() {
        ValueType::List(item_type) => Ok(Box::new(TakeList {
            target,
            limit,
            items_type: item_type.clone(),
        })),
        ValueType::String => Ok(Box::new(TakeString { target, limit })),
        _ => Err(FindItError::BadExpression(
            "Take method can only be applied to String or List types".to_string(),
        )),
    }
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
    fn test_simple_take() -> Result<(), FindItError> {
        let expr = read_expr("\"abc\".take(2)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::String("ab".into()));

        Ok(())
    }

    #[test]
    fn test_take_large_number() -> Result<(), FindItError> {
        let expr = read_expr("\"abc\".take(100)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::String("abc".into()));

        Ok(())
    }

    #[test]
    fn take_return_type() -> Result<(), FindItError> {
        let expr = read_expr("\"abc\".take(2)")?;

        assert_eq!(expr.expected_type(), ValueType::String);

        Ok(())
    }

    #[test]
    fn length_no_string_take() {
        let err = read_expr("12.take(2)").err();
        assert!(err.is_some())
    }

    #[test]
    fn length_no_number_take() {
        let err = read_expr("\"abc\".take(\"a\")").err();
        assert!(err.is_some())
    }

    #[test]
    fn test_take_empty_string() -> Result<(), FindItError> {
        let expr = read_expr("content.take(100)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn test_take_empty_number() -> Result<(), FindItError> {
        let expr = read_expr("\"abc\".take(size)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }
    #[test]
    fn test_simple_take_list() -> Result<(), FindItError> {
        let expr = read_expr("[1, 2, 3].take(2)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(
            expr.eval(file),
            Value::List(List::new_eager(
                Rc::new(ValueType::Number),
                vec![Value::Number(1), Value::Number(2),].into_iter(),
            ))
        );

        Ok(())
    }

    #[test]
    fn test_take_list_large_number() -> Result<(), FindItError> {
        let expr = read_expr("[1, 2, 3].take(100)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(
            expr.eval(file),
            Value::List(List::new_eager(
                Rc::new(ValueType::Number),
                vec![Value::Number(1), Value::Number(2), Value::Number(3),].into_iter(),
            ))
        );

        Ok(())
    }

    #[test]
    fn take_list_return_type() -> Result<(), FindItError> {
        let expr = read_expr("[1, 2, 3].take(2)")?;

        assert_eq!(
            expr.expected_type(),
            ValueType::List(Rc::new(ValueType::Number))
        );

        Ok(())
    }

    #[test]
    fn take_list_nan_error() {
        let err = read_expr("[1, 2, 3].take(\"a\")").err();
        assert!(err.is_some())
    }

    #[test]
    fn test_take_list_empty_string() -> Result<(), FindItError> {
        let expr = read_expr("files.take(100)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn test_take_list_empty_number() -> Result<(), FindItError> {
        let expr = read_expr("[1, 3].take(size)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }
}
