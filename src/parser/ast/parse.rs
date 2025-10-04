use crate::parser::ast::expression::Expression;

#[derive(Debug, PartialEq)]
pub(crate) struct Parse {
    pub(crate) str: Box<Expression>,
    pub(crate) format: Box<Expression>,
}
