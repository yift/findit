use crate::parser::ast::expression::Expression;

#[derive(Debug, PartialEq)]
pub(crate) struct IsCheck {
    pub(crate) expression: Box<Expression>,
    pub(crate) check_type: IsType,
    pub(crate) negate: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum IsType {
    True,
    False,
    None,
    Some,
}
