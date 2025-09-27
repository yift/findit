use std::{fmt::Display, iter::Peekable};

use crate::parser::{
    expression::{Expression, ParserError, build_expression_with_priority},
    lexer::LexerItem,
    tokens::Token,
};

#[derive(Debug, PartialEq)]
pub(crate) struct Position {
    pub(crate) sub_string: Box<Expression>,
    pub(crate) super_string: Box<Expression>,
}

impl Position {
    pub(crate) fn new(sub_string: Expression, super_string: Expression) -> Self {
        Self {
            sub_string: Box::new(sub_string),
            super_string: Box::new(super_string),
        }
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "POSITION({} IN {})", self.sub_string, self.super_string,)
    }
}

pub(super) fn build_position(
    lex: &mut Peekable<impl Iterator<Item = LexerItem>>,
) -> Result<Expression, ParserError> {
    let Some(open) = lex.next() else {
        return Err(ParserError::UnexpectedEof);
    };
    if open.token != Token::OpenBrackets {
        return Err(ParserError::UnexpectedToken(open.span));
    }
    let sub_string = build_expression_with_priority(lex, 0, |f| f == Some(&Token::In))?;
    lex.next();
    let super_string =
        build_expression_with_priority(lex, 0, |f| f == Some(&Token::CloseBrackets))?;
    lex.next();
    Ok(Expression::Position(Position::new(
        sub_string,
        super_string,
    )))
}
