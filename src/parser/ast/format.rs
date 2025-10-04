use crate::parser::ast::expression::Expression;

#[derive(Debug, PartialEq)]
pub(crate) struct Format {
    pub(crate) timestamp: Box<Expression>,
    pub(crate) format: Box<Expression>,
}
