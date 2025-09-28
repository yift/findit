#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum ArithmeticOperator {
    Plus,
    Minus,
    Multiply,
    Divide,
    Module,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum LogicalOperator {
    And,
    Or,
    Xor,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum ComparisonOperator {
    Eq,
    Neq,
    LargerThen,
    LargerThenEq,
    SmallerThen,
    SmallerThenEq,
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum BitwiseOperator {
    And,
    Or,
    Xor,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum BinaryOperator {
    Arithmetic(ArithmeticOperator),
    Logical(LogicalOperator),
    Comparison(ComparisonOperator),
    BitwiseOperator(BitwiseOperator),
    Matches,
    Of,
    Dot,
}
