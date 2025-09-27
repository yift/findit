use std::fmt::Display;

use crate::parser::{expression::Expression, operator::BinaryOperator};

#[derive(Debug, PartialEq)]
pub(crate) struct BinaryExpression {
    pub(crate) left: Box<Expression>,
    pub(crate) operator: BinaryOperator,
    pub(crate) right: Box<Expression>,
}

impl BinaryExpression {
    pub(crate) fn new(left: Expression, operator: BinaryOperator, right: Expression) -> Self {
        BinaryExpression {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        }
    }
}
impl Display for BinaryExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{} {} {}]", self.left, self.operator, self.right)
    }
}
