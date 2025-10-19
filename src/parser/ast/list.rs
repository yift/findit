use crate::parser::ast::expression::Expression;

#[derive(Debug, PartialEq)]
pub(crate) struct List {
    pub(crate) items: Vec<Expression>,
}
