use std::iter::Peekable;

use crate::parser::{
    expression::{Expression, ParserError, build_expression_with_priority},
    lexer::LexerItem,
    span::Span,
    tokens::Token,
};

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

impl Case {
    pub(crate) fn new(branches: Vec<CaseBranch>, default_outcome: Option<Expression>) -> Self {
        Self {
            branches,
            default_outcome: default_outcome.map(Box::new),
        }
    }
}
pub(super) fn build_case(
    lex: &mut Peekable<impl Iterator<Item = LexerItem>>,
    case_span: &Span,
) -> Result<Expression, ParserError> {
    let mut branches = vec![];
    let mut default_outcome = None;

    loop {
        let Some(next) = lex.next() else {
            return Err(ParserError::UnexpectedEof);
        };
        match next.token {
            Token::When => {
                let condition =
                    build_expression_with_priority(lex, 0, |f| f == Some(&Token::Then))?;
                lex.next();
                let outcome = build_expression_with_priority(lex, 0, |f| {
                    f == Some(&Token::When) || f == Some(&Token::End) || f == Some(&Token::Else)
                })?;
                branches.push(CaseBranch::new(condition, outcome));
            }
            Token::End => {
                break;
            }
            Token::Else => {
                default_outcome = Some(build_expression_with_priority(lex, 0, |f| {
                    f == Some(&Token::End)
                })?);
                lex.next();
                break;
            }
            _ => return Err(ParserError::UnexpectedToken(next.span)),
        }
    }
    if branches.is_empty() {
        return Err(ParserError::NoBranches(*case_span));
    }
    Ok(Expression::Case(Case::new(branches, default_outcome)))
}
