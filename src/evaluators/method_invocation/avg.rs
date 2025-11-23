use crate::{
    errors::FindItError,
    evaluators::expr::Evaluator,
    file_wrapper::FileWrapper,
    value::{Value, ValueType},
};
use std::ops::{Add, Deref};

#[derive(Default)]
struct AvgCalc {
    total: u64,
    count: u64,
}
impl Add<u64> for AvgCalc {
    type Output = Self;
    fn add(self, rhs: u64) -> Self {
        Self {
            total: self.total + rhs,
            count: self.count + 1,
        }
    }
}
impl From<AvgCalc> for Value {
    fn from(value: AvgCalc) -> Self {
        if value.count == 0 {
            Value::Empty
        } else {
            Value::Number(value.total / value.count)
        }
    }
}

struct Avg {
    target: Box<dyn Evaluator>,
}
impl Evaluator for Avg {
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
            .fold(AvgCalc::default(), |acc, item| {
                if let Value::Number(n) = item {
                    acc + n
                } else {
                    acc
                }
            })
            .into()
    }
}

pub(super) fn new_avg(target: Box<dyn Evaluator>) -> Result<Box<dyn Evaluator>, FindItError> {
    let ValueType::List(item_type) = target.expected_type() else {
        return Err(FindItError::BadExpression(
            "Avg method can only be applied to a List of numbers".to_string(),
        ));
    };
    if item_type.deref() != &ValueType::Number {
        return Err(FindItError::BadExpression(
            "Avg method can only be applied to List of Number type".to_string(),
        ));
    }
    Ok(Box::new(Avg { target }))
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
    fn test_simple_avg() -> Result<(), FindItError> {
        let expr = read_expr(":[10, 20, 50, 30, 40].avg()")?;
        let file = &FileWrapper::new(PathBuf::new(), 1);

        assert_eq!(expr.eval(file), Value::Number(30));

        Ok(())
    }

    #[test]
    fn test_empty_avg() -> Result<(), FindItError> {
        let expr = read_expr(":[10, 20, 50, 30, 40].filter($n $n < 5).avg()")?;
        let file = &FileWrapper::new(PathBuf::new(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn test_avg_expected_type() -> Result<(), FindItError> {
        let expr = read_expr(":[1, 2, 3, 4, 5, 6].avg()")?;

        assert_eq!(expr.expected_type(), ValueType::Number);

        Ok(())
    }

    #[test]
    fn test_avg_nop_return_empty() -> Result<(), FindItError> {
        let expr = read_expr("files.map($f $f.length()).avg()")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn test_avg_ignores_non_numbers() -> Result<(), FindItError> {
        let expr = read_expr("files.map($f ($f/ \"first-229.txt\").length()).avg()")?;
        let path = Path::new("tests/test_cases/filter/test_files");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Number(66));

        Ok(())
    }

    #[test]
    fn length_no_list_avg() {
        let err = read_expr("12.avg()").err();
        assert!(err.is_some())
    }

    #[test]
    fn length_no_number_avg() {
        let err = read_expr(":[\"a\", \"b\"].avg()").err();
        assert!(err.is_some())
    }
}
