use crate::parser::ast::expression::Expression;

#[derive(Debug, PartialEq)]
pub(crate) struct Position {
    pub(crate) sub_string: Box<Expression>,
    pub(crate) super_string: Box<Expression>,
}
