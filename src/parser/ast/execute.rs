use crate::parser::ast::expression::Expression;

#[derive(Debug, PartialEq)]
pub(crate) struct SpawnOrExecute {
    pub(crate) spawn: bool,
    pub(crate) bin: Box<Expression>,
    pub(crate) args: Vec<Expression>,
    pub(crate) into: Option<Box<Expression>>,
}
