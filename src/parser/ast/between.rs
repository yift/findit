use crate::parser::ast::expression::Expression;

#[derive(Debug, PartialEq)]
pub(crate) struct Between {
    pub(crate) reference: Box<Expression>,
    pub(crate) lower_limit: Box<Expression>,
    pub(crate) upper_limit: Box<Expression>,
}
