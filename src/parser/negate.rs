use crate::parser::expression::Expression;

#[derive(Debug, PartialEq)]
pub(crate) struct Negate {
    pub(crate) expression: Box<Expression>,
}
impl Negate {
    pub(crate) fn new(expression: Expression) -> Self {
        Self {
            expression: Box::new(expression),
        }
    }
}
