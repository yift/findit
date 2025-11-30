use crate::parser::ast::expression::Expression;

#[derive(Debug, PartialEq)]
pub(crate) struct Field {
    pub(crate) name: String,
    pub(crate) value: Expression,
}

#[derive(Debug, PartialEq)]
pub(crate) struct ClassDefinition {
    pub(crate) fields: Vec<Field>,
}

#[derive(Debug, PartialEq)]
pub(crate) struct ClassAccess {
    pub(crate) target: Box<Expression>,
    pub(crate) field: String,
}
