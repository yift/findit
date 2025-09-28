use crate::parser::ast::expression::Expression;

#[derive(Debug, PartialEq)]
pub(crate) struct If {
    pub(crate) condition: Box<Expression>,
    pub(crate) then_branch: Box<Expression>,
    pub(crate) else_branch: Option<Box<Expression>>,
}
