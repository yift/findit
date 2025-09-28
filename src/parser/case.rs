use std::iter::Peekable;

use crate::parser::{
    ast::{
        case::{Case, CaseBranch},
        expression::Expression,
    },
    expression::build_expression_with_priority,
    lexer::LexerItem,
    parser_error::ParserError,
    span::Span,
    tokens::Token,
};

impl Case {
    pub(super) fn new(branches: Vec<CaseBranch>, default_outcome: Option<Expression>) -> Self {
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
