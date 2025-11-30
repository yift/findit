use std::collections::HashMap;

use crate::{
    errors::FindItError,
    file_wrapper::FileWrapper,
    order::{OrderDirection, OrderItem},
    parser::{
        ast::expression::Expression, ast::order_by::OrderByDirection, parse_expression,
        parse_order_by,
    },
    value::{Value, ValueType},
};

pub(crate) trait Evaluator {
    fn eval(&self, file: &FileWrapper) -> Value;
    fn expected_type(&self) -> ValueType;
}

#[derive(Debug, Default)]
pub(crate) struct BindingsTypes {
    types: HashMap<String, (usize, ValueType)>,
    max_index: usize,
}
impl BindingsTypes {
    pub(crate) fn get(&self, name: &str) -> Result<(&usize, &ValueType), FindItError> {
        let Some((index, tp)) = self.types.get(name) else {
            return Err(FindItError::BadExpression(format!(
                "Can not find binding name: '{name}'"
            )));
        };
        Ok((index, tp))
    }
    pub(crate) fn with(&self, name: &str, tp: ValueType) -> Self {
        let mut types = self.types.clone();
        types.insert(name.to_string(), (self.max_index, tp));
        Self {
            types,
            max_index: self.max_index + 1,
        }
    }
}
pub(crate) trait EvaluatorFactory {
    fn build(&self, bindings: &BindingsTypes) -> Result<Box<dyn Evaluator>, FindItError>;
}

impl EvaluatorFactory for Expression {
    fn build(&self, bindings: &BindingsTypes) -> Result<Box<dyn Evaluator>, FindItError> {
        match self {
            Expression::Literal(val) => Ok(val.into()),
            Expression::Binary(bin) => bin.build(bindings),
            Expression::Negate(exp) => exp.build(bindings),
            Expression::Brackets(expr) => expr.build(bindings),
            Expression::Access(access) => Ok(access.into()),
            Expression::IsCheck(is_check) => is_check.build(bindings),
            Expression::If(iff) => iff.build(bindings),
            Expression::Case(case) => case.build(bindings),
            Expression::Between(between) => between.build(bindings),
            Expression::Position(position) => position.build(bindings),
            Expression::Function(func) => func.build(bindings),
            Expression::SpawnOrExecute(spawn_or_exec) => spawn_or_exec.build(bindings),
            Expression::SelfDivide(self_divide) => self_divide.build(bindings),
            Expression::Cast(a) => a.build(bindings),
            Expression::Format(f) => f.build(bindings),
            Expression::Parse(p) => p.build(bindings),
            Expression::Replace(p) => p.build(bindings),
            Expression::BindingReplacement(b) => b.build(bindings),
            Expression::With(w) => w.build(bindings),
            Expression::List(l) => l.build(bindings),
            Expression::MethodInvocation(l) => l.build(bindings),
            Expression::ClassDefinition(d) => d.build(bindings),
            Expression::ClassAccess(a) => a.build(bindings),
        }
    }
}

pub(crate) fn read_expr(expr: &str) -> Result<Box<dyn Evaluator>, FindItError> {
    let expression = parse_expression(expr)?;

    expression.build(&BindingsTypes::default())
}

pub(crate) fn read_order_by(sql: &str) -> Result<Vec<OrderItem>, FindItError> {
    let order_by = parse_order_by(sql)?;

    let mut order = vec![];
    let types = BindingsTypes::default();
    for item in order_by.items {
        let evaluator = item.expression.build(&types)?;
        let direction = match &item.direction {
            OrderByDirection::Asc => OrderDirection::Asc,
            OrderByDirection::Desc => OrderDirection::Desc,
        };
        order.push(OrderItem {
            direction,
            evaluator,
        });
    }

    Ok(order)
}

#[cfg(test)]
mod tests {
    use std::{env, path::Path};

    use super::*;

    #[test]
    fn compound_for_not_file_return_empty() -> Result<(), FindItError> {
        let sql = "(parent / \"no_such_file.ext\").name";
        let eval = read_expr(sql)?;
        let file = Path::new("/").to_path_buf();
        let wrapper = FileWrapper::new(file, 1);
        let value = eval.eval(&wrapper);

        assert!(value == Value::Empty);
        Ok(())
    }

    #[test]
    fn is_true_without_bool() {
        let sql = "name IS TRUE";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }

    #[test]
    fn is_false_returns_bool() {
        let sql = "TRUE IS FALSE";
        let expr = read_expr(sql).unwrap();

        assert_eq!(expr.expected_type(), ValueType::Bool);
    }

    #[test]
    fn is_null_returns_bool() {
        let sql = "123 IS NONE";
        let expr = read_expr(sql).unwrap();

        assert_eq!(expr.expected_type(), ValueType::Bool);
    }

    #[test]
    fn between_expr_must_have_same_type_as_min() {
        let sql = "count BETWEEN \"a\" AND 10";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }

    #[test]
    fn between_expr_must_have_same_type_as_max() {
        let sql = "count BETWEEN 1 AND \"b\"";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }

    #[test]
    fn between_expr_must_have_a_value() -> Result<(), FindItError> {
        let sql = "content BETWEEN \"a\" AND \"b\"";
        let eval = read_expr(sql)?;
        let file = env::current_dir()?;
        let wrapper = FileWrapper::new(file, 1);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Empty);

        Ok(())
    }

    #[test]
    fn between_min_must_have_a_value() -> Result<(), FindItError> {
        let sql = "\"a\" BETWEEN content AND \"b\"";
        let eval = read_expr(sql)?;
        let file = env::current_dir()?;
        let wrapper = FileWrapper::new(file, 1);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Empty);

        Ok(())
    }

    #[test]
    fn between_max_must_have_a_value() -> Result<(), FindItError> {
        let sql = "\"c\" BETWEEN \"b\" AND content";
        let eval = read_expr(sql)?;
        let file = env::current_dir()?;
        let wrapper = FileWrapper::new(file, 1);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Empty);

        Ok(())
    }

    #[test]
    fn between_expect_bool() -> Result<(), FindItError> {
        let sql = "\"a\" BETWEEN \"b\" AND \"c\"";
        let eval = read_expr(sql)?;

        assert_eq!(eval.expected_type(), ValueType::Bool);

        Ok(())
    }
}
