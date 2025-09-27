use std::{fmt::Display, iter::Peekable};

use crate::parser::{
    expression::{Expression, ParserError, build_expression_with_priority},
    lexer::LexerItem,
    tokens::Token,
};

#[derive(Debug, PartialEq)]
pub(crate) struct Substring {
    pub(crate) super_string: Box<Expression>,
    pub(crate) substring_from: Option<Box<Expression>>,
    pub(crate) substring_for: Option<Box<Expression>>,
}

impl Substring {
    pub(crate) fn new(
        super_string: Expression,
        substring_from: Option<Expression>,
        substring_for: Option<Expression>,
    ) -> Self {
        Self {
            super_string: Box::new(super_string),
            substring_for: substring_for.map(Box::new),
            substring_from: substring_from.map(Box::new),
        }
    }
}

impl Display for Substring {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SUBSTRING({} ", self.super_string)?;
        if let Some(substring_from) = &self.substring_from {
            write!(f, "FROM {}", substring_from)?;
        };
        if let Some(substring_for) = &self.substring_for {
            write!(f, "FROM {}", substring_for)?;
        };
        write!(f, ")")
    }
}
pub(super) fn build_substring(
    lex: &mut Peekable<impl Iterator<Item = LexerItem>>,
) -> Result<Expression, ParserError> {
    let Some(open) = lex.next() else {
        return Err(ParserError::UnexpectedEof);
    };
    if open.token != Token::OpenBrackets {
        return Err(ParserError::UnexpectedToken(open.span));
    };
    let super_string = build_expression_with_priority(lex, 0, |f| {
        f == Some(&Token::From) || f == Some(&Token::For)
    })?;
    let Some(next) = lex.next() else {
        return Err(ParserError::UnexpectedEof);
    };
    if next.token == Token::From {
        let substring_from = build_expression_with_priority(lex, 0, |f| {
            f == Some(&Token::CloseBrackets) || f == Some(&Token::For)
        })?;
        let Some(next) = lex.next() else {
            return Err(ParserError::UnexpectedEof);
        };
        if next.token == Token::CloseBrackets {
            Ok(Expression::Substring(Substring::new(
                super_string,
                Some(substring_from),
                None,
            )))
        } else {
            let substring_for =
                build_expression_with_priority(lex, 0, |f| f == Some(&Token::CloseBrackets))?;
            lex.next();
            Ok(Expression::Substring(Substring::new(
                super_string,
                Some(substring_from),
                Some(substring_for),
            )))
        }
    } else {
        let substring_for = build_expression_with_priority(lex, 0, |f| {
            f == Some(&Token::CloseBrackets) || f == Some(&Token::From)
        })?;
        let Some(next) = lex.next() else {
            return Err(ParserError::UnexpectedEof);
        };
        if next.token == Token::CloseBrackets {
            Ok(Expression::Substring(Substring::new(
                super_string,
                None,
                Some(substring_for),
            )))
        } else {
            let substring_from =
                build_expression_with_priority(lex, 0, |f| f == Some(&Token::CloseBrackets))?;
            lex.next();
            Ok(Expression::Substring(Substring::new(
                super_string,
                Some(substring_from),
                Some(substring_for),
            )))
        }
    }
}
