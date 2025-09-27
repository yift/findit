use std::iter::Peekable;

use crate::parser::{
    expression::{Expression, ParserError, build_expression_with_priority},
    lexer::LexerItem,
    tokens::Token,
};

#[derive(Debug, PartialEq)]
pub(crate) struct If {
    pub(crate) condition: Box<Expression>,
    pub(crate) then_branch: Box<Expression>,
    pub(crate) else_branch: Option<Box<Expression>>,
}

impl If {
    pub(crate) fn new(
        condition: Expression,
        then_branch: Expression,
        else_branch: Option<Expression>,
    ) -> Self {
        Self {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch: else_branch.map(Box::new),
        }
    }
}

pub(super) fn build_if(
    lex: &mut Peekable<impl Iterator<Item = LexerItem>>,
) -> Result<Expression, ParserError> {
    let condition = build_expression_with_priority(lex, 0, |f| f == Some(&Token::Then))?;
    lex.next();
    let then = build_expression_with_priority(lex, 0, |f| {
        f == Some(&Token::End) || f == Some(&Token::Else)
    })?;
    let has_else = lex
        .next()
        .map(|n| n.token == Token::Else)
        .unwrap_or_default();
    let else_branch = if has_else {
        let branch = build_expression_with_priority(lex, 0, |f| f == Some(&Token::End))?;
        lex.next();
        Some(branch)
    } else {
        None
    };
    Ok(Expression::If(If::new(condition, then, else_branch)))
}
