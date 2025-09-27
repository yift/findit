use crate::parser::expression::Expression;

#[derive(Debug, PartialEq)]
pub(crate) struct SelfDivide {
    pub(crate) expression: Box<Expression>,
}
impl SelfDivide {
    pub(crate) fn new(expression: Expression) -> Self {
        Self {
            expression: Box::new(expression),
        }
    }
}
