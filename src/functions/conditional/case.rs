use sqlparser::ast::{CaseWhen, Expr};

use crate::{
    errors::FindItError,
    expr::{Evaluator, get_eval},
    file_wrapper::FileWrapper,
    value::{Value, ValueType},
};

struct Condition {
    condition: Box<dyn Evaluator>,
    result: Box<dyn Evaluator>,
}

impl TryFrom<&CaseWhen> for Condition {
    type Error = FindItError;
    fn try_from(value: &CaseWhen) -> Result<Self, Self::Error> {
        let condition = get_eval(&value.condition)?;
        let result = get_eval(&value.result)?;
        Ok(Self { condition, result })
    }
}

struct Case {
    branches: Vec<Condition>,
    default: Option<Box<dyn Evaluator>>,
    value_type: ValueType,
}

impl Evaluator for Case {
    fn eval(&self, file: &FileWrapper) -> Value {
        for c in &self.branches {
            if c.condition.eval(file) == Value::Bool(true) {
                return c.result.eval(file);
            }
        }
        match &self.default {
            Option::Some(d) => d.eval(file),
            None => Value::Empty,
        }
    }
    fn expected_type(&self) -> ValueType {
        self.value_type
    }
}

pub(crate) fn new_case(
    conditions: &[CaseWhen],
    else_result: &Option<Box<Expr>>,
) -> Result<Box<dyn Evaluator>, FindItError> {
    let mut value_type = ValueType::Empty;
    let mut branches = vec![];
    for b in conditions {
        let b: Condition = b.try_into()?;
        let expected_type = b.result.expected_type();
        if expected_type != ValueType::Empty && value_type == ValueType::Empty {
            value_type = expected_type;
        } else if expected_type != ValueType::Empty && expected_type != value_type {
            return Err(FindItError::BadExpression(
                "CASE should result in the same type for all the branches".into(),
            ));
        }

        branches.push(b);
    }
    let default = match else_result {
        None => None,
        Some(d) => {
            let d = get_eval(d)?;
            if d.expected_type() != ValueType::Empty
                && value_type != ValueType::Empty
                && d.expected_type() != value_type
            {
                return Err(FindItError::BadExpression(
                    "CASE else should result in the same type as all the branches".into(),
                ));
            }
            Some(d)
        }
    };
    Ok(Box::new(Case {
        branches,
        default,
        value_type,
    }))
}

#[cfg(test)]
mod tests {
    use crate::{expr::read_expr, value::ValueType};

    #[test]
    fn test_case_with_different_result_type() {
        let sql = "CASE WHEN extension = 'txt' THEN 'a' WHEN extension = 'b' THEN 'c' WHEN extension = 'bash' THEN 4 ELSE 'DIRECTORY' END";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }

    #[test]
    fn test_case_with_different_default_type() {
        let sql = "CASE WHEN extension = 'txt' THEN 'a' WHEN extension = 'b' THEN 'c' WHEN extension = 'bash' THEN 'd' ELSE 4 END";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }

    #[test]
    fn test_case_expected_type() {
        let sql = "CASE WHEN extension = 'txt' THEN 'a' WHEN extension = 'b' THEN 'c' END";
        let expr = read_expr(sql).unwrap();

        assert_eq!(expr.expected_type(), ValueType::String);
    }

    #[test]
    fn test_case_with_operand() {
        let sql = "CASE parent WHEN extension = 'txt' THEN 'a' WHEN extension = 'b' THEN 'c' END";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }
}
