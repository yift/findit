use crate::parser::ast::expression::Expression;

#[derive(Debug, PartialEq)]
pub(crate) struct SelfDivide {
    pub(crate) expression: Box<Expression>,
}
