use crate::{
    parser::{
        ast::binary_expression::BinaryExpression,
        ast::function::Function,
        ast::if_expression::If,
        ast::is_check::IsCheck,
        ast::negate::Negate,
        ast::position::Position,
        ast::self_divide::SelfDivide,
        ast::substr::Substring,
        ast::{access::Access, between::Between, case::Case, execute::SpawnOrExecute},
    },
    value::Value,
};

#[derive(Debug, PartialEq)]
pub(crate) enum Expression {
    Literal(Value),
    Binary(BinaryExpression),
    Negate(Negate),
    Brackets(Box<Expression>),
    Access(Access),
    IsCheck(IsCheck),
    If(If),
    Case(Case),
    Between(Between),
    Position(Position),
    Substring(Substring),
    Function(Function),
    SpawnOrExecute(SpawnOrExecute),
    SelfDivide(SelfDivide),
}
