use std::{fmt::Display, iter::Peekable};

use crate::parser::{
    expression::{Expression, ParserError, build_expression_with_priority},
    lexer::LexerItem,
    operator::{BinaryOperator, LogicalOperator},
    tokens::Token,
};

#[derive(Debug, PartialEq)]
pub(crate) struct Between {
    pub(crate) reference: Box<Expression>,
    pub(crate) lower_limit: Box<Expression>,
    pub(crate) upper_limit: Box<Expression>,
}

impl Between {
    pub(crate) fn new(
        reference: Expression,
        lower_limit: Expression,
        upper_limit: Expression,
    ) -> Self {
        Self {
            reference: Box::new(reference),
            lower_limit: Box::new(lower_limit),
            upper_limit: Box::new(upper_limit),
        }
    }
}

impl Display for Between {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} BETWEEN {} AND {}",
            self.reference, self.lower_limit, self.upper_limit
        )
    }
}

pub(super) fn build_between(
    reference: Expression,
    lex: &mut Peekable<impl Iterator<Item = LexerItem>>,
) -> Result<Expression, ParserError> {
    lex.next();
    let lower_limit = build_expression_with_priority(lex, 0, |f| {
        f == Some(&Token::BinaryOperator(BinaryOperator::Logical(
            LogicalOperator::And,
        )))
    })?;
    lex.next();
    let upper_limit = build_expression_with_priority(lex, 10, |f| f.is_none())?;
    Ok(Expression::Between(Between::new(
        reference,
        lower_limit,
        upper_limit,
    )))
}
