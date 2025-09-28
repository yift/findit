use crate::parser::ast::expression::Expression;

#[derive(Debug, PartialEq)]
pub(crate) enum OrderByDirection {
    Asc,
    Desc,
}

#[derive(Debug, PartialEq)]
pub(crate) struct OrderByItem {
    pub(crate) expression: Expression,
    pub(crate) direction: OrderByDirection,
}

#[derive(Debug, PartialEq)]
pub(crate) struct OrderByExpression {
    pub(crate) items: Vec<OrderByItem>,
}
