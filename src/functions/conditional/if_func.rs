use crate::{
    errors::FindItError,
    expr::Evaluator,
    file_wrapper::FileWrapper,
    value::{Value, ValueType},
};

struct If {
    condition: Box<dyn Evaluator>,
    positive: Box<dyn Evaluator>,
    negative: Box<dyn Evaluator>,
}
impl Evaluator for If {
    fn eval(&self, file: &FileWrapper) -> Value {
        match self.condition.eval(file) {
            Value::Bool(true) => self.positive.eval(file),
            Value::Bool(false) => self.negative.eval(file),
            _ => Value::Empty,
        }
    }
    fn expected_type(&self) -> ValueType {
        self.positive.expected_type()
    }
}
pub(crate) fn build_if(
    mut args: Vec<Box<dyn Evaluator>>,
) -> Result<Box<dyn Evaluator>, FindItError> {
    let negative = args
        .pop()
        .ok_or_else(|| FindItError::BadExpression("IF must have a negative result.".into()))?;
    let positive = args
        .pop()
        .ok_or_else(|| FindItError::BadExpression("IF must have a positive result.".into()))?;
    let condition = args
        .pop()
        .ok_or_else(|| FindItError::BadExpression("IF must have a condition.".into()))?;
    if !args.is_empty() {
        return Err(FindItError::BadExpression(
            "IF must have 3 arguments.".into(),
        ));
    }
    if condition.expected_type() != ValueType::Bool {
        return Err(FindItError::BadExpression(
            "IF condition must be boolean.".into(),
        ));
    }
    if negative.expected_type() != positive.expected_type() {
        return Err(FindItError::BadExpression(
            "IF results must be the same.".into(),
        ));
    }

    Ok(Box::new(If {
        condition,
        negative,
        positive,
    }))
}

#[cfg(test)]
mod tests {
    use crate::{expr::read_expr, value::ValueType};

    #[test]
    fn test_if_with_a_lot_or_args() {
        let sql = "IF(true, 1, 2, 3, 4, 5, 6)";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }

    #[test]
    fn test_if_with_two_args() {
        let sql = "IF(true, 1 )";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }

    #[test]
    fn test_if_with_one_args() {
        let sql = "IF(true)";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }

    #[test]
    fn test_if_with_no_args() {
        let sql = "IF()";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }

    #[test]
    fn test_if_with_different_type() {
        let sql = "IF(TRUE, 1, 'one')";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }

    #[test]
    fn test_if_expected_value() {
        let sql = "IF(TRUE, 1, 10)";
        let expr = read_expr(sql).unwrap();

        assert_eq!(expr.expected_type(), ValueType::Number);
    }

    #[test]
    fn test_if_with_numeric_condition() {
        let sql = "IF(200, 'no', 'one')";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }
}
