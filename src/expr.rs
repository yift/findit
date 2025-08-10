use std::ops::Deref;

use sqlparser::{
    ast::{Expr, Ident, SetExpr, Spanned, Statement},
    dialect::GenericDialect,
    parser::Parser,
};

use crate::{errors::FindItError, extract::get_extractor, file_wrapper::FileWrapper, value::Value};

pub(crate) trait Evaluator {
    fn eval(&self, file: &FileWrapper) -> Value;
}

fn get_eval(expr: &Expr) -> Result<Box<dyn Evaluator>, FindItError> {
    match expr {
        Expr::Identifier(ident) => get_extractor(ident),
        Expr::CompoundIdentifier(identifiers) => new_compound_eval(identifiers),
        _ => Err(FindItError::BadExpression("QQQ".into())),
    }
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
    let next = new_compound_eval(&identifiers[1..])?;
    Ok(Box::new(CompoundEval { evaluator, next }))
}

impl Evaluator for CompoundEval {
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::Path(path) = self.evaluator.eval(file) else {
            return Value::Empty;
        };
        let file = FileWrapper::new(path, file.dept() + 1);
        self.next.eval(&file)
    }
}

pub(crate) fn read_expr(sql: &str) -> Result<Box<dyn Evaluator>, FindItError> {
    let my_sql = format!("SELECT * FROM table_name WHERE \n{}\n;", sql);
    let dialect = GenericDialect {};
    let ast = Parser::parse_sql(&dialect, &my_sql)?;
    if ast.len() != 1 {
        return Err(FindItError::BadFilter(sql.to_string()));
    }
    let ast = ast.first().unwrap();
    let Statement::Query(select) = ast else {
        return Err(FindItError::BadFilter(sql.to_string()));
    };
    let SetExpr::Select(select) = select.body.deref() else {
        return Err(FindItError::BadFilter(sql.to_string()));
    };

    let Some(filter) = &select.selection else {
        return Err(FindItError::BadFilter(sql.to_string()));
    };
    if filter.span().start.line != 2 || filter.span().start.column != 1 {
        return Err(FindItError::BadFilter(sql.to_string()));
    }
    if filter.span().end != ast.span().end {
        return Err(FindItError::BadFilter(sql.to_string()));
    }

    get_eval(filter)
}
