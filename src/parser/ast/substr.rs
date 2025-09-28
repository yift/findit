use crate::parser::ast::expression::Expression;

#[derive(Debug, PartialEq)]
pub(crate) struct Substring {
    pub(crate) super_string: Box<Expression>,
    pub(crate) substring_from: Option<Box<Expression>>,
    pub(crate) substring_for: Option<Box<Expression>>,
}
