use std::iter::Peekable;

use crate::parser::{
    ast::{execute::SpawnOrExecute, expression::Expression},
    expression::build_expression_with_priority,
    lexer::LexerItem,
    parser_error::ParserError,
    tokens::Token,
};

impl SpawnOrExecute {
    pub(super) fn new(
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

#[cfg(test)]
mod tests {
    use crate::parser::parse_expression;

    #[test]
    fn test_spawn_just_spawn() {
        let source = "spawn";
        let err = parse_expression(source).err();

        assert!(err.is_some());
    }
    #[test]
    fn test_spawn_with_no_open_brackets() {
        let source = "spawn 3";
        let err = parse_expression(source).err();

        assert!(err.is_some());
    }
}
