use crate::{
    parser::ast::{
        access::Access, as_cast::As, between::Between, binary_expression::BinaryExpression,
        binding::Binding, case::Case, execute::SpawnOrExecute, format::Format, function::Function,
        if_expression::If, is_check::IsCheck, list::List, negate::Negate, parse::Parse,
        position::Position, replace::Replace, self_divide::SelfDivide, substr::Substring,
        with::With,
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
    Format(Format),
    Parse(Parse),
    Substring(Substring),
    Function(Function),
    SpawnOrExecute(SpawnOrExecute),
    SelfDivide(SelfDivide),
    Cast(As),
    Replace(Replace),
    BindingReplacement(Binding),
    With(With),
    List(List),
}
