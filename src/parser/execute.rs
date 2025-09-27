use std::iter::Peekable;

use crate::parser::{
    expression::{Expression, ParserError, build_expression_with_priority},
    lexer::LexerItem,
    tokens::Token,
};

#[derive(Debug, PartialEq)]
pub(crate) struct SpawnOrExecute {
    pub(crate) spawn: bool,
    pub(crate) bin: Box<Expression>,
    pub(crate) args: Vec<Expression>,
    pub(crate) into: Option<Box<Expression>>,
}

impl SpawnOrExecute {
    pub(crate) fn new(
        spawn: bool,
        bin: Expression,
        args: Vec<Expression>,
        into: Option<Expression>,
    ) -> Self {
        Self {
            spawn,
            bin: Box::new(bin),
            args,
            into: into.map(Box::new),
        }
    }
}
pub(super) fn build_spawn_or_exec(
    spawn: bool,
    lex: &mut Peekable<impl Iterator<Item = LexerItem>>,
) -> Result<Expression, ParserError> {
    let Some(open) = lex.next() else {
        return Err(ParserError::UnexpectedEof);
    };
    if open.token != Token::OpenBrackets {
        return Err(ParserError::UnexpectedToken(open.span));
    };
    let bin = build_expression_with_priority(lex, 0, |f| {
        f == Some(&Token::CloseBrackets) || f == Some(&Token::Comma) || f == Some(&Token::Into)
    })?;
    if let Some(next) = lex.peek()
        && next.token == Token::Comma
    {
        lex.next();
    }
    let mut args = vec![];
    let next = loop {
        if let Some(next) = lex.peek()
            && (next.token == Token::CloseBrackets || next.token == Token::Into)
        {
            break next;
        }
        let arg = build_expression_with_priority(lex, 0, |f| {
            f == Some(&Token::CloseBrackets) || f == Some(&Token::Comma) || f == Some(&Token::Into)
        })?;
        args.push(arg);
        if let Some(next) = lex.peek()
            && next.token == Token::Comma
        {
            lex.next();
        }
    };
    let into = if next.token == Token::Into {
        lex.next();
        Some(build_expression_with_priority(lex, 0, |f| {
            f == Some(&Token::CloseBrackets)
        })?)
    } else {
        None
    };
    lex.next();
    Ok(Expression::SpawnOrExecute(SpawnOrExecute::new(
        spawn, bin, args, into,
    )))
}
