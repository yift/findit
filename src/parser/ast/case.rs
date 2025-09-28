use crate::parser::ast::expression::Expression;

#[derive(Debug, PartialEq)]
pub(crate) struct CaseBranch {
    pub(crate) condition: Box<Expression>,
    pub(crate) outcome: Box<Expression>,
}
impl CaseBranch {
    pub(crate) fn new(condition: Expression, outcome: Expression) -> Self {
        Self {
            condition: Box::new(condition),
            outcome: Box::new(outcome),
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct Case {
    pub(crate) branches: Vec<CaseBranch>,
    pub(crate) default_outcome: Option<Box<Expression>>,
}
