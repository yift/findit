use crate::parser::{ast::expression::Expression, ast::function_name::FunctionName};

#[derive(Debug, PartialEq)]
pub(crate) struct Function {
    pub(crate) name: FunctionName,
    pub(crate) args: Vec<Expression>,
}
