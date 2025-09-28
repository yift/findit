use crate::{
    between::new_between,
    errors::FindItError,
    extract::get_extractor,
    file_wrapper::FileWrapper,
    functions::{
        conditional::{case::new_case, if_func::build_if},
        into::new_function,
    },
    is_check::new_is_check,
    literal_value::new_literal_value,
    order::{OrderDirection, OrderItem},
    parser::{
        ast::expression::Expression, ast::order_by::OrderByDirection, parse_expression,
        parse_order_by,
    },
    string_functions::{new_position, new_substring},
    value::{Value, ValueType},
};

pub(crate) trait Evaluator {
    fn eval(&self, file: &FileWrapper) -> Value;
    fn expected_type(&self) -> ValueType;
}

pub(crate) fn get_eval(expr: &Expression) -> Result<Box<dyn Evaluator>, FindItError> {
    match expr {
        Expression::Literal(val) => Ok(new_literal_value(val)),
        Expression::Binary(bin) => bin.try_into(),
        Expression::Negate(exp) => exp.try_into(),
        Expression::Brackets(expr) => get_eval(expr),
        Expression::Access(access) => Ok(get_extractor(access)),
        Expression::IsCheck(is_check) => new_is_check(is_check),
        Expression::If(iff) => build_if(iff),
        Expression::Case(case) => new_case(case),
        Expression::Between(between) => new_between(between),
        Expression::Position(position) => new_position(position),
        Expression::Substring(substring) => new_substring(substring),
        Expression::Function(func) => new_function(func),
        Expression::SpawnOrExecute(spawn_or_exec) => spawn_or_exec.try_into(),
        Expression::SelfDivide(self_divide) => self_divide.try_into(),
    }
}

pub(crate) fn read_expr(expr: &str) -> Result<Box<dyn Evaluator>, FindItError> {
    let expression = parse_expression(expr)?;
    get_eval(&expression)
}

pub(crate) fn read_order_by(sql: &str) -> Result<Vec<OrderItem>, FindItError> {
    let order_by = parse_order_by(sql)?;

    let mut order = vec![];
    for item in order_by.items {
        let evaluator = get_eval(&item.expression)?;
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
