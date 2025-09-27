use std::fmt::Display;

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

impl Display for BinaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOperator::Arithmetic(o) => o.fmt(f),
            BinaryOperator::Comparison(o) => o.fmt(f),
            BinaryOperator::Logical(o) => o.fmt(f),
            BinaryOperator::BitwiseOperator(o) => o.fmt(f),
            BinaryOperator::Matches => write!(f, "Matches"),
            BinaryOperator::Of => write!(f, "OF"),
            BinaryOperator::Dot => write!(f, "."),
        }
    }
}

impl Display for ArithmeticOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArithmeticOperator::Divide => write!(f, "/"),
            ArithmeticOperator::Multiply => write!(f, "*"),
            ArithmeticOperator::Plus => write!(f, "+"),
            ArithmeticOperator::Minus => write!(f, "-"),
            ArithmeticOperator::Module => write!(f, "%"),
        }
    }
}
impl Display for LogicalOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogicalOperator::And => write!(f, "AND"),
            LogicalOperator::Or => write!(f, "OR"),
            LogicalOperator::Xor => write!(f, "XOR"),
        }
    }
}
impl Display for ComparisonOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComparisonOperator::Eq => write!(f, "="),
            ComparisonOperator::Neq => write!(f, "!="),
            ComparisonOperator::LargerThen => write!(f, ">"),
            ComparisonOperator::SmallerThen => write!(f, "<"),
            ComparisonOperator::LargerThenEq => write!(f, ">="),
            ComparisonOperator::SmallerThenEq => write!(f, "<="),
        }
    }
}
impl Display for BitwiseOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BitwiseOperator::And => write!(f, "&"),
            BitwiseOperator::Or => write!(f, "|"),
            BitwiseOperator::Xor => write!(f, "^"),
        }
    }
}
