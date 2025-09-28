use crate::parser::ast::expression::Expression;

#[derive(Debug, PartialEq)]
pub(crate) struct As {
    pub(crate) expression: Box<Expression>,
    pub(crate) cast_type: CastType,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum CastType {
    Bool,
    String,
    Number,
    Date,
    Path,
}
