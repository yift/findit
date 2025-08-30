use sqlparser::{
    ast::{AccessExpr, Expr, Ident, OrderByKind, SetExpr, Statement},
    dialect::GenericDialect,
    parser::Parser,
};

use crate::{
    binary_operator::new_binary_operator,
    errors::FindItError,
    extract::get_extractor,
    file_wrapper::FileWrapper,
    literal_value::new_literal_value,
    order::{OrderDirection, OrderItem},
    string_functions::{new_position, new_regex, new_substring, new_trim},
    unary_operators::new_unary_operator,
    value::{Value, ValueType},
};

pub(crate) trait Evaluator {
    fn eval(&self, file: &FileWrapper) -> Value;
    fn expected_type(&self) -> ValueType;
}

pub(crate) fn get_eval(expr: &Expr) -> Result<Box<dyn Evaluator>, FindItError> {
    match expr {
        Expr::Identifier(ident) => get_extractor(ident),
        Expr::CompoundIdentifier(identifiers) => new_compound_eval(identifiers),
        Expr::IsTrue(expr) => new_is_true_false(expr, false, false),
        Expr::IsNotTrue(expr) => new_is_true_false(expr, true, true),
        Expr::IsFalse(expr) => new_is_true_false(expr, true, false),
        Expr::IsNotFalse(expr) => new_is_true_false(expr, false, true),
        Expr::IsNull(expr) => new_is_null(expr, false),
        Expr::IsNotNull(expr) => new_is_null(expr, true),
        Expr::Between {
            expr,
            negated,
            low,
            high,
        } => new_between(expr, low, high, *negated),
        Expr::BinaryOp { left, op, right } => new_binary_operator(left, op, right),
        Expr::Value(val) => new_literal_value(&val.value),
        Expr::Nested(expr) => get_eval(expr),
        Expr::CompoundFieldAccess { root, access_chain } => {
            new_compound_field_access(root, access_chain)
        }
        Expr::SimilarTo {
            negated,
            expr,
            pattern,
            escape_char,
        } => {
            if escape_char.is_some() {
                Err(FindItError::BadExpression(
                    "SIMILAR TO with escape character".into(),
                ))
            } else {
                new_regex(expr, pattern, *negated)
            }
        }
        Expr::RLike {
            negated,
            expr,
            pattern,
            regexp: _,
        } => new_regex(expr, pattern, *negated),
        Expr::UnaryOp { op, expr } => new_unary_operator(expr, op),
        Expr::Position { expr, r#in } => new_position(r#in, expr),
        Expr::Substring {
            expr,
            substring_from,
            substring_for,
            special: _,
            shorthand: _,
        } => new_substring(expr, substring_from, substring_for),
        Expr::Trim {
            expr,
            trim_where,
            trim_what,
            trim_characters,
        } => {
            if trim_characters.is_some() {
                Err(FindItError::BadExpression(
                    "TRIM with trim characters".into(),
                ))
            } else {
                new_trim(expr, trim_where, trim_what)
            }
        }
        _ => {
            dbg!(expr);
            Err(FindItError::BadExpression(format!("{expr}")))
        }
    }
    /*
    /// A literal value, such as string, number, date or NULL
    /// Scalar function call e.g. `LEFT(foo, 5)`
    Function(Function),
    /// `CASE [<operand>] WHEN <condition> THEN <result> ... [ELSE <result>] END`
    ///
    /// Note we only recognize a complete single expression as `<condition>`,
    /// not `< 0` nor `1, 2, 3` as allowed in a `<simple when clause>` per
    /// <https://jakewheat.github.io/sql-overview/sql-2011-foundation-grammar.html#simple-when-clause>
    Case {
        case_token: AttachedToken,
        end_token: AttachedToken,
        operand: Option<Box<Expr>>,
        conditions: Vec<CaseWhen>,
        else_result: Option<Box<Expr>>,
    },

     */
}
struct CompoundEval {
    evaluator: Box<dyn Evaluator>,
    next: Box<dyn Evaluator>,
}
fn new_compound_eval(identifiers: &[Ident]) -> Result<Box<dyn Evaluator>, FindItError> {
    let Some(first) = identifiers.first() else {
        return Err(FindItError::BadExpression(
            "Empty compound identifiers".into(),
        ));
    };
    let evaluator = get_extractor(first)?;
    if identifiers.len() == 1 {
        return Ok(evaluator);
    }
    if evaluator.expected_type() != ValueType::Path {
        return Err(FindItError::BadExpression(
            "compound identifier must return a path".into(),
        ));
    }
    let next = new_compound_eval(&identifiers[1..])?;
    Ok(Box::new(CompoundEval { evaluator, next }))
}
fn new_compound_field_access(
    root: &Expr,
    access_chain: &[AccessExpr],
) -> Result<Box<dyn Evaluator>, FindItError> {
    let first = get_eval(root)?;
    let Some(next) = access_chain.first() else {
        return Ok(first);
    };
    let AccessExpr::Dot(next) = next else {
        return Err(FindItError::BadExpression(
            "Only dot compound access is allowed".into(),
        ));
    };
    let next = new_compound_field_access(next, &access_chain[1..])?;
    Ok(Box::new(CompoundEval {
        evaluator: first,
        next,
    }))
}

impl Evaluator for CompoundEval {
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::Path(path) = self.evaluator.eval(file) else {
            return Value::Empty;
        };
        let file = FileWrapper::new(path, file.dept() + 1);
        self.next.eval(&file)
    }

    fn expected_type(&self) -> ValueType {
        self.next.expected_type()
    }
}

struct IsTrueFalse {
    evaluator: Box<dyn Evaluator>,
    negate: bool,
    default: bool,
}
fn new_is_true_false(
    expr: &Expr,
    negate: bool,
    default: bool,
) -> Result<Box<dyn Evaluator>, FindItError> {
    let evaluator = get_eval(expr)?;
    if evaluator.expected_type() != ValueType::Bool {
        return Err(FindItError::BadExpression(
            "IS (NOT) TRUE/FALSE must refer to a Boolean".into(),
        ));
    }
    Ok(Box::new(IsTrueFalse {
        evaluator,
        negate,
        default,
    }))
}

impl Evaluator for IsTrueFalse {
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::Bool(val) = self.evaluator.eval(file) else {
            return Value::Bool(self.default);
        };

        Value::Bool(val ^ self.negate)
    }

    fn expected_type(&self) -> ValueType {
        ValueType::Bool
    }
}

struct IsNull {
    evaluator: Box<dyn Evaluator>,
    negate: bool,
}
fn new_is_null(expr: &Expr, negate: bool) -> Result<Box<dyn Evaluator>, FindItError> {
    let evaluator = get_eval(expr)?;
    Ok(Box::new(IsNull { evaluator, negate }))
}

impl Evaluator for IsNull {
    fn eval(&self, file: &FileWrapper) -> Value {
        if self.evaluator.eval(file) == Value::Empty {
            Value::Bool(!self.negate)
        } else {
            Value::Bool(self.negate)
        }
    }

    fn expected_type(&self) -> ValueType {
        ValueType::Bool
    }
}

struct Between {
    evaluator: Box<dyn Evaluator>,
    low: Box<dyn Evaluator>,
    high: Box<dyn Evaluator>,
    negate: bool,
}
fn new_between(
    expr: &Expr,
    low: &Expr,
    high: &Expr,
    negate: bool,
) -> Result<Box<dyn Evaluator>, FindItError> {
    let evaluator = get_eval(expr)?;
    let low = get_eval(low)?;
    if evaluator.expected_type() != low.expected_type() {
        return Err(FindItError::BadExpression(
            "Between low must have the same type as the expression".into(),
        ));
    }
    let high = get_eval(high)?;
    if evaluator.expected_type() != high.expected_type() {
        return Err(FindItError::BadExpression(
            "Between high must have the same type as the expression".into(),
        ));
    }
    Ok(Box::new(Between {
        evaluator,
        low,
        high,
        negate,
    }))
}

impl Evaluator for Between {
    fn eval(&self, file: &FileWrapper) -> Value {
        let value = self.evaluator.eval(file);
        if value == Value::Empty {
            return Value::Empty;
        }
        let low = self.low.eval(file);
        if low == Value::Empty {
            return Value::Empty;
        }
        if value < low {
            return Value::Bool(self.negate);
        }
        let high = self.high.eval(file);
        if high == Value::Empty {
            return Value::Empty;
        }
        if value > high {
            Value::Bool(self.negate)
        } else {
            Value::Bool(!self.negate)
        }
    }

    fn expected_type(&self) -> ValueType {
        ValueType::Bool
    }
}

pub(crate) fn read_expr(sql: &str) -> Result<Box<dyn Evaluator>, FindItError> {
    let my_sql = format!(
        "SELECT * FROM table_name WHERE \n{sql}\n GROUP BY col HAVING col ORDER BY col LIMIT 20;"
    );
    let dialect = GenericDialect {};
    let ast = Parser::parse_sql(&dialect, &my_sql)?;
    if ast.len() != 1 {
        return Err(FindItError::BadFilter(sql.to_string()));
    }
    let ast = ast.first().unwrap();
    let Statement::Query(select) = ast else {
        return Err(FindItError::BadFilter(sql.to_string()));
    };
    let SetExpr::Select(select) = &*select.body else {
        return Err(FindItError::BadFilter(sql.to_string()));
    };

    let Some(filter) = &select.selection else {
        return Err(FindItError::BadFilter(sql.to_string()));
    };

    get_eval(filter)
}

pub(crate) fn read_order_by(sql: &str) -> Result<Vec<OrderItem>, FindItError> {
    let my_sql = format!("SELECT * FROM table_name ORDER BY \n{sql}\nLIMIT 1;");
    let dialect = GenericDialect {};
    let ast = Parser::parse_sql(&dialect, &my_sql)?;
    if ast.len() != 1 {
        return Err(FindItError::BadOrderBy(sql.to_string()));
    }
    let ast = ast.first().unwrap();
    let Statement::Query(select) = ast else {
        return Err(FindItError::BadOrderBy(sql.to_string()));
    };
    let Some(order_by) = &select.order_by else {
        return Err(FindItError::BadOrderBy(sql.to_string()));
    };
    if order_by.interpolate.is_some() {
        return Err(FindItError::BadOrderBy(sql.to_string()));
    }
    let OrderByKind::Expressions(order_by) = &order_by.kind else {
        return Err(FindItError::BadOrderBy(sql.to_string()));
    };

    let mut order = vec![];
    for item in order_by {
        if item.with_fill.is_some() {
            return Err(FindItError::BadOrderBy(sql.to_string()));
        }
        let evaluator = get_eval(&item.expr)?;
        let direction = match item.options.asc {
            None => OrderDirection::Asc,
            Some(true) => OrderDirection::Asc,
            Some(false) => OrderDirection::Desc,
        };
        let nulls_first = match item.options.nulls_first {
            None => false,
            Some(true) => true,
            Some(false) => false,
        };
        order.push(OrderItem {
            direction,
            evaluator,
            nulls_first,
        });
    }

    Ok(order)
}

#[cfg(test)]
mod tests {
    use std::{env, path::Path};

    use super::*;

    #[test]
    fn test_similar_to_with_escape_character() {
        let sql = "name  SIMILAR TO 'val'  ESCAPE 'e'";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }

    #[test]
    fn test_trim_with_characters() {
        let sql = "TRIM('a', 'b')";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }

    #[test]
    fn compound_not_for_a_file() {
        let sql = "name.extension";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }

    #[test]
    fn compound_with_subscript() {
        let sql = "name['extension']";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }

    #[test]
    fn compound_for_not_file_return_empty() -> Result<(), FindItError> {
        let sql = "(parent / 'no_such_file.ext').name";
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
        let sql = "is_link IS FALSE";
        let expr = read_expr(sql).unwrap();

        assert_eq!(expr.expected_type(), ValueType::Bool);
    }

    #[test]
    fn is_null_returns_bool() {
        let sql = "is_link IS NULL";
        let expr = read_expr(sql).unwrap();

        assert_eq!(expr.expected_type(), ValueType::Bool);
    }

    #[test]
    fn between_expr_must_have_same_type_as_min() {
        let sql = "count BETWEEN 'a' AND 10";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }

    #[test]
    fn between_expr_must_have_same_type_as_max() {
        let sql = "count BETWEEN 1 AND 'b'";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }

    #[test]
    fn between_expr_must_have_a_value() -> Result<(), FindItError> {
        let sql = "content BETWEEN 'a' AND 'b'";
        let eval = read_expr(sql)?;
        let file = env::current_dir()?;
        let wrapper = FileWrapper::new(file, 1);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Empty);

        Ok(())
    }

    #[test]
    fn between_min_must_have_a_value() -> Result<(), FindItError> {
        let sql = "'a' BETWEEN content AND 'b'";
        let eval = read_expr(sql)?;
        let file = env::current_dir()?;
        let wrapper = FileWrapper::new(file, 1);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Empty);

        Ok(())
    }

    #[test]
    fn between_max_must_have_a_value() -> Result<(), FindItError> {
        let sql = "'c' BETWEEN 'b' AND content";
        let eval = read_expr(sql)?;
        let file = env::current_dir()?;
        let wrapper = FileWrapper::new(file, 1);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Empty);

        Ok(())
    }

    #[test]
    fn between_expect_bool() -> Result<(), FindItError> {
        let sql = "'a' BETWEEN 'b' AND 'c'";
        let eval = read_expr(sql)?;

        assert_eq!(eval.expected_type(), ValueType::Bool);

        Ok(())
    }
}
