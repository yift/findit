use crate::{
    errors::FindItError,
    evaluators::expr::{BindingsTypes, Evaluator, EvaluatorFactory},
    file_wrapper::FileWrapper,
    parser::ast::case::{Case as CaseExpression, CaseBranch},
    value::{Value, ValueType},
};

struct Condition {
    condition: Box<dyn Evaluator>,
    result: Box<dyn Evaluator>,
}

impl CaseBranch {
    fn build(&self, bindings: &BindingsTypes) -> Result<Condition, FindItError> {
        let condition = self.condition.build(bindings)?;
        let result = self.outcome.build(bindings)?;
        Ok(Condition { condition, result })
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
        self.value_type.clone()
    }
}

impl EvaluatorFactory for CaseExpression {
    fn build(&self, bindings: &BindingsTypes) -> Result<Box<dyn Evaluator>, FindItError> {
        let mut value_type = ValueType::Empty;
        let mut branches = vec![];
        for b in &self.branches {
            let b = b.build(bindings)?;
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
        let default = match &self.default_outcome {
            None => None,
            Some(d) => {
                let d = d.build(bindings)?;
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
}

#[cfg(test)]
mod tests {
    use crate::{evaluators::expr::read_expr, value::ValueType};

    #[test]
    fn test_case_with_different_result_type() {
        let sql = "CASE WHEN extension = \"txt\" THEN \"a\" WHEN extension = \"b\" THEN \"c\" WHEN extension = \"bash\" THEN 4 ELSE \"DIRECTORY\" END";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }

    #[test]
    fn test_case_with_different_default_type() {
        let sql = "CASE WHEN extension = \"txt\" THEN \"a\" WHEN extension = \"b\" THEN \"c\" WHEN extension = \"bash\" THEN \"d\" ELSE 4 END";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }

    #[test]
    fn test_case_expected_type() {
        let sql = "CASE WHEN extension = \"txt\" THEN \"a\" WHEN extension = \"b\" THEN \"c\" END";
        let expr = read_expr(sql).unwrap();

        assert_eq!(expr.expected_type(), ValueType::String);
    }

    #[test]
    fn test_case_with_operand() {
        let sql =
            "CASE parent WHEN extension = \"txt\" THEN \"a\" WHEN extension = \"b\" THEN \"c\" END";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }
}
