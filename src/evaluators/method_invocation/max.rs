use crate::{
    errors::FindItError,
    evaluators::expr::Evaluator,
    file_wrapper::FileWrapper,
    value::{Value, ValueType},
};
use std::ops::Deref;

struct Max {
    target: Box<dyn Evaluator>,
    item_type: ValueType,
}
impl Evaluator for Max {
    fn expected_type(&self) -> ValueType {
        self.item_type.clone()
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::List(value) = self.target.eval(file) else {
            return Value::Empty;
        };
        value.items().into_iter().max().unwrap_or(Value::Empty)
    }
}

pub(super) fn new_max(target: Box<dyn Evaluator>) -> Result<Box<dyn Evaluator>, FindItError> {
    let ValueType::List(item_type) = target.expected_type() else {
        return Err(FindItError::BadExpression(
            "Max method can only be applied to a List".to_string(),
        ));
    };
    let item_type = item_type.deref().clone();
    Ok(Box::new(Max { target, item_type }))
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use crate::{
        errors::FindItError,
        evaluators::expr::read_expr,
        file_wrapper::FileWrapper,
        value::{Value, ValueType},
    };

    #[test]
    fn test_simple_max() -> Result<(), FindItError> {
        let expr = read_expr(":[1, 2, 3, 1].max()")?;
        let file = &FileWrapper::new(PathBuf::new(), 1);

        assert_eq!(expr.eval(file), Value::Number(3));

        Ok(())
    }

    #[test]
    fn test_max_expected_type() -> Result<(), FindItError> {
        let expr = read_expr(":[1, 2, 3, 4, 5, 6].max()")?;

        assert_eq!(expr.expected_type(), ValueType::Number);

        Ok(())
    }

    #[test]
    fn test_max_nop_return_empty() -> Result<(), FindItError> {
        let expr = read_expr("files.map({f} {f}.length()).max()")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn test_max_empty_list_return_nothing() -> Result<(), FindItError> {
        let expr = read_expr(":[1, 3, 4].filter({n} {n} > 10).max()")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn no_list_max() {
        let err = read_expr("12.max()").err();
        assert!(err.is_some())
    }
}
