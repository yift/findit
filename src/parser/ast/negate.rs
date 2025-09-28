use crate::parser::ast::expression::Expression;

#[derive(Debug, PartialEq)]
pub(crate) struct Negate {
    pub(crate) expression: Box<Expression>,
}
