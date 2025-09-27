use crate::{
    errors::FindItError,
    expr::{Evaluator, get_eval},
    file_wrapper::FileWrapper,
    parser::if_expression::If as IfExpression,
    value::{Value, ValueType},
};

struct NoElseIf {
    condition: Box<dyn Evaluator>,
    then: Box<dyn Evaluator>,
}
struct IfWithElse {
    condition: Box<dyn Evaluator>,
    then: Box<dyn Evaluator>,
    else_branch: Box<dyn Evaluator>,
}

impl Evaluator for NoElseIf {
    fn eval(&self, file: &FileWrapper) -> Value {
        match self.condition.eval(file) {
            Value::Bool(true) => self.then.eval(file),
            _ => Value::Empty,
        }
    }
    fn expected_type(&self) -> ValueType {
        self.then.expected_type()
    }
}

impl Evaluator for IfWithElse {
    fn eval(&self, file: &FileWrapper) -> Value {
        match self.condition.eval(file) {
            Value::Bool(true) => self.then.eval(file),
            Value::Bool(false) => self.else_branch.eval(file),
            _ => Value::Empty,
        }
    }
    fn expected_type(&self) -> ValueType {
        self.then.expected_type()
    }
}

pub(crate) fn build_if(iff: &IfExpression) -> Result<Box<dyn Evaluator>, FindItError> {
    let condition = get_eval(&iff.condition)?;
    if condition.expected_type() != ValueType::Bool {
        return Err(FindItError::BadExpression(
            "IF condition must be boolean.".into(),
        ));
    }
    let then = get_eval(&iff.then_branch)?;
    if let Some(else_branch) = &iff.else_branch {
        let else_branch = get_eval(else_branch)?;
        if else_branch.expected_type() != then.expected_type() {
            return Err(FindItError::BadExpression(
                "IF results must be the same.".into(),
            ));
        }
        Ok(Box::new(IfWithElse {
            condition,
            then,
            else_branch,
        }))
    } else {
        Ok(Box::new(NoElseIf { condition, then }))
    }
}

#[cfg(test)]
mod tests {
    use crate::{expr::read_expr, value::ValueType};

    #[test]
    fn test_if_with_different_type() {
        let sql = "IF TRUE THEN 1 ELSE \"one\" END";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }

    #[test]
    fn test_if_expected_value() {
        let sql = "IF TRUE THEN 1 ELSE 10  END";
        let expr = read_expr(sql).unwrap();

        assert_eq!(expr.expected_type(), ValueType::Number);
    }

    #[test]
    fn test_if_with_numeric_condition() {
        let sql = "IF 200 THEN \"no\" ELSE \"one\" END";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }
}
