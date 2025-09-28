use crate::parser::{ast::expression::Expression, ast::operator::BinaryOperator};

#[derive(Debug, PartialEq)]
pub(crate) struct BinaryExpression {
    pub(crate) left: Box<Expression>,
    pub(crate) operator: BinaryOperator,
    pub(crate) right: Box<Expression>,
}
