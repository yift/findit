use crate::parser::{
    ast::binary_expression::BinaryExpression, ast::expression::Expression,
    ast::operator::BinaryOperator,
};

impl BinaryExpression {
    pub(super) fn new(left: Expression, operator: BinaryOperator, right: Expression) -> Self {
        BinaryExpression {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        }
    }
}
