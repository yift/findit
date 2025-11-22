use crate::{
    errors::FindItError,
    evaluators::expr::Evaluator,
    file_wrapper::FileWrapper,
    value::{Value, ValueType},
};
use std::ops::Deref;

struct Min {
    target: Box<dyn Evaluator>,
    item_type: ValueType,
}
impl Evaluator for Min {
    fn expected_type(&self) -> ValueType {
        self.item_type.clone()
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::List(value) = self.target.eval(file) else {
            return Value::Empty;
        };
        value.items().into_iter().min().unwrap_or(Value::Empty)
    }
}

pub(super) fn new_min(target: Box<dyn Evaluator>) -> Result<Box<dyn Evaluator>, FindItError> {
    let ValueType::List(item_type) = target.expected_type() else {
        return Err(FindItError::BadExpression(
            "Min method can only be applied to a List".to_string(),
        ));
    };
    let item_type = item_type.deref().clone();
    Ok(Box::new(Min { target, item_type }))
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
    fn test_simple_min() -> Result<(), FindItError> {
        let expr = read_expr(":[10, 2, 3, 10].min()")?;
        let file = &FileWrapper::new(PathBuf::new(), 1);

        assert_eq!(expr.eval(file), Value::Number(2));

        Ok(())
    }

    #[test]
    fn test_min_expected_type() -> Result<(), FindItError> {
        let expr = read_expr(":[1, 2, 3, 4, 5, 6].min()")?;

        assert_eq!(expr.expected_type(), ValueType::Number);

        Ok(())
    }

    #[test]
    fn test_min_nop_return_empty() -> Result<(), FindItError> {
        let expr = read_expr("files.map({f} {f}.length()).min()")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn test_min_empty_list_return_nothing() -> Result<(), FindItError> {
        let expr = read_expr(":[1, 3, 4].filter({n} {n} > 10).min()")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn no_list_min() {
        let err = read_expr("12.min()").err();
        assert!(err.is_some())
    }
}
