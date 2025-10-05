use crate::parser::ast::expression::Expression;

#[derive(Debug, PartialEq)]
pub(crate) struct With {
    pub(crate) names: Vec<(String, Box<Expression>)>,
    pub(crate) action: Box<Expression>,
}
