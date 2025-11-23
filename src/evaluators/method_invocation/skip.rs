use std::rc::Rc;

use crate::{
    errors::FindItError,
    evaluators::expr::{BindingsTypes, Evaluator, EvaluatorFactory},
    file_wrapper::FileWrapper,
    parser::ast::expression::Expression,
    value::{List, Value, ValueType},
};

struct SkipString {
    target: Box<dyn Evaluator>,
    by: Box<dyn Evaluator>,
}
impl Evaluator for SkipString {
    fn expected_type(&self) -> ValueType {
        ValueType::String
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::String(target_value) = self.target.eval(file) else {
            return Value::Empty;
        };
        let Value::Number(by_value) = self.by.eval(file) else {
            return Value::Empty;
        };
        target_value
            .chars()
            .skip(by_value as usize)
            .collect::<String>()
            .into()
    }
}

struct SkipList {
    target: Box<dyn Evaluator>,
    by: Box<dyn Evaluator>,
    items_type: Rc<ValueType>,
}
impl Evaluator for SkipList {
    fn expected_type(&self) -> ValueType {
        ValueType::List(self.items_type.clone())
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::List(target_value) = self.target.eval(file) else {
            return Value::Empty;
        };
        let Value::Number(by_value) = self.by.eval(file) else {
            return Value::Empty;
        };
        let iter = target_value.items().into_iter().skip(by_value as usize);
        Value::List(List::new_lazy(self.items_type.clone(), iter))
    }
}

pub(super) fn new_skip(
    target: Box<dyn Evaluator>,
    by: &Expression,
    bindings: &BindingsTypes,
) -> Result<Box<dyn Evaluator>, FindItError> {
    let by = by.build(bindings)?;

    if by.expected_type() != ValueType::Number {
        return Err(FindItError::BadExpression(
            "Skip method argument must be a Number".to_string(),
        ));
    }
    match target.expected_type() {
        ValueType::List(item_type) => Ok(Box::new(SkipList {
            target,
            by,
            items_type: item_type.clone(),
        })),
        ValueType::String => Ok(Box::new(SkipString { target, by })),
        _ => Err(FindItError::BadExpression(
            "Skip method can only be applied to String or List types".to_string(),
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
    fn test_simple_skip() -> Result<(), FindItError> {
        let expr = read_expr("\"abc\".skip(2)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::String("c".into()));

        Ok(())
    }

    #[test]
    fn test_skip_large_number() -> Result<(), FindItError> {
        let expr = read_expr("\"abc\".skip(100)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::String("".into()));

        Ok(())
    }

    #[test]
    fn test_skip_empty_string() -> Result<(), FindItError> {
        let expr = read_expr("content.skip(100)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn test_skip_empty_number() -> Result<(), FindItError> {
        let expr = read_expr("\"abc\".skip(size)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn skip_return_type() -> Result<(), FindItError> {
        let expr = read_expr("\"abc\".skip(2)")?;

        assert_eq!(expr.expected_type(), ValueType::String);

        Ok(())
    }

    #[test]
    fn length_no_string_skip() {
        let err = read_expr("12.skip(2)").err();
        assert!(err.is_some())
    }

    #[test]
    fn length_no_number_skip() {
        let err = read_expr("\"abc\".skip(\"a\")").err();
        assert!(err.is_some())
    }

    #[test]
    fn test_skip_list() -> Result<(), FindItError> {
        let expr = read_expr("[1, 2, 4, 5].skip(2)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(
            expr.eval(file),
            Value::List(List::new_eager(
                Rc::new(ValueType::Number),
                vec![Value::Number(4), Value::Number(5),].into_iter(),
            ))
        );

        Ok(())
    }

    #[test]
    fn test_skip_list_large_number() -> Result<(), FindItError> {
        let expr = read_expr("[1, 2, 4, 5].skip(100)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(
            expr.eval(file),
            Value::List(List::new_eager(
                Rc::new(ValueType::Number),
                vec![].into_iter(),
            ))
        );

        Ok(())
    }

    #[test]
    fn skip_list_return_type() -> Result<(), FindItError> {
        let expr = read_expr("[1, 2, 4, 5].skip(2)")?;

        assert_eq!(
            expr.expected_type(),
            ValueType::List(Rc::new(ValueType::Number))
        );

        Ok(())
    }

    #[test]
    fn skip_list_nan() {
        let err = read_expr("[1, 2, 4, 5].skip(\"a\")").err();
        assert!(err.is_some())
    }

    #[test]
    fn test_skip_list_empty_string() -> Result<(), FindItError> {
        let expr = read_expr("files.skip(100)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn test_skip_list_empty_number() -> Result<(), FindItError> {
        let expr = read_expr("[1, 3].skip(size)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }
}
