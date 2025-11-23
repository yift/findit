use crate::{
    errors::FindItError,
    evaluators::expr::Evaluator,
    file_wrapper::FileWrapper,
    value::{Value, ValueType},
};
use std::ops::Deref;

struct Sum {
    target: Box<dyn Evaluator>,
}
impl Evaluator for Sum {
    fn expected_type(&self) -> ValueType {
        ValueType::Number
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::List(value) = self.target.eval(file) else {
            return Value::Empty;
        };
        value
            .items()
            .into_iter()
            .fold(0u64, |acc, item| {
                if let Value::Number(n) = item {
                    acc + n
                } else {
                    acc
                }
            })
            .into()
    }
}

pub(super) fn new_sum(target: Box<dyn Evaluator>) -> Result<Box<dyn Evaluator>, FindItError> {
    let ValueType::List(item_type) = target.expected_type() else {
        return Err(FindItError::BadExpression(
            "Sum method can only be applied to a List of numbers".to_string(),
        ));
    };
    if item_type.deref() != &ValueType::Number {
        return Err(FindItError::BadExpression(
            "Sum method can only be applied to List of Number type".to_string(),
        ));
    }
    Ok(Box::new(Sum { target }))
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
    fn test_simple_sum() -> Result<(), FindItError> {
        let expr = read_expr(":[1, 2, 3].sum()")?;
        let file = &FileWrapper::new(PathBuf::new(), 1);

        assert_eq!(expr.eval(file), Value::Number(6));

        Ok(())
    }

    #[test]
    fn test_sum_expected_type() -> Result<(), FindItError> {
        let expr = read_expr(":[1, 2, 3, 4, 5, 6].sum()")?;

        assert_eq!(expr.expected_type(), ValueType::Number);

        Ok(())
    }

    #[test]
    fn test_sum_nop_return_empty() -> Result<(), FindItError> {
        let expr = read_expr("files.map($f $f.length()).sum()")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn test_sum_ignores_non_numbers() -> Result<(), FindItError> {
        let expr = read_expr("files.map($f ($f/ \"first-229.txt\").length()).sum()")?;
        let path = Path::new("tests/test_cases/filter/test_files");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Number(66));

        Ok(())
    }

    #[test]
    fn length_no_list_sum() {
        let err = read_expr("12.sum()").err();
        assert!(err.is_some())
    }

    #[test]
    fn length_no_number_sum() {
        let err = read_expr(":[\"a\", \"b\"].sum()").err();
        assert!(err.is_some())
    }
}
