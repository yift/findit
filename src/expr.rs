use sqlparser::{
    ast::{AccessExpr, Expr, GroupByExpr, Ident, SetExpr, Statement},
    dialect::GenericDialect,
    parser::Parser,
};

use crate::{
    binary_operator::new_binary_operator,
    errors::FindItError,
    extract::get_extractor,
    file_wrapper::FileWrapper,
    literal_value::new_literal_value,
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
    /// ```sql
    /// TRIM([BOTH | LEADING | TRAILING] [<expr> FROM] <expr>)
    /// TRIM(<expr>)
    /// TRIM(<expr>, [, characters]) -- only Snowflake or Bigquery
    /// ```
    Trim {
        expr: Box<Expr>,
        // ([BOTH | LEADING | TRAILING]
        trim_where: Option<TrimWhereField>,
        trim_what: Option<Box<Expr>>,
        trim_characters: Option<Vec<Expr>>,
    },
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
    let my_sql = format!("SELECT * FROM table_name WHERE \n{sql}\n;");
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
    let GroupByExpr::Expressions(exs, mods) = &select.group_by else {
        return Err(FindItError::BadFilter(sql.to_string()));
    };
    if !exs.is_empty() && !mods.is_empty() {
        return Err(FindItError::BadFilter(sql.to_string()));
    }
    if !select.sort_by.is_empty() {
        return Err(FindItError::BadFilter(sql.to_string()));
    }

    let Some(filter) = &select.selection else {
        return Err(FindItError::BadFilter(sql.to_string()));
    };

    get_eval(filter)
}
