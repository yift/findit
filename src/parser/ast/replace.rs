use crate::parser::ast::expression::Expression;

#[derive(Debug, PartialEq)]
pub(crate) enum ReplaceWhat {
    Pattern(Box<Expression>),
    String(Box<Expression>),
}
#[derive(Debug, PartialEq)]
pub(crate) struct Replace {
    pub(crate) source: Box<Expression>,
    pub(crate) what: ReplaceWhat,
    pub(crate) to: Box<Expression>,
}
