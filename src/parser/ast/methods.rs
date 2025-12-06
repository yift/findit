use crate::parser::ast::expression::Expression;

#[derive(Debug, PartialEq)]
pub(crate) struct LambdaFunction {
    pub(crate) parameter: String,
    pub(crate) body: Box<Expression>,
}

#[derive(Debug, PartialEq)]
pub(crate) enum Method {
    Length,
    ToUpper,
    ToLower,
    Trim,
    TrimHead,
    TrimTail,
    Reverse,
    Map(LambdaFunction),
    Filter(LambdaFunction),
    Sum,
    Max,
    Min,
    Avg,
    Sort,
    SortBy(LambdaFunction),
    Distinct,
    DistinctBy(LambdaFunction),
    Skip(Box<Expression>),
    Take(Box<Expression>),
    Join(Option<Box<Expression>>),
    Split(Box<Expression>),
    Lines,
    Words,
    First,
    Last,
    Contains(Box<Expression>),
    IndexOf(Box<Expression>),
    FlatMap(LambdaFunction),
    All(LambdaFunction),
    Any(LambdaFunction),
    GroupBy(LambdaFunction),
}

#[derive(Debug, PartialEq)]
pub(crate) struct MethodInvocation {
    pub(crate) target: Option<Box<Expression>>,
    pub(crate) method: Method,
}
