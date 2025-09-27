use std::iter::Peekable;

use crate::parser::{
    expression::{Expression, ParserError, build_expression_with_priority},
    function_name::FunctionName,
    lexer::LexerItem,
    tokens::Token,
};

#[derive(Debug, PartialEq)]
pub(crate) struct Function {
    pub(crate) name: FunctionName,
    pub(crate) args: Vec<Expression>,
}

impl Function {
    pub(crate) fn new(name: FunctionName, args: Vec<Expression>) -> Self {
        Self { name, args }
    }
}
pub(super) fn build_function(
    name: FunctionName,
    lex: &mut Peekable<impl Iterator<Item = LexerItem>>,
) -> Result<Expression, ParserError> {
    let Some(open) = lex.next() else {
        return Err(ParserError::UnexpectedEof);
    };
    if open.token != Token::OpenBrackets {
        return Err(ParserError::UnexpectedToken(open.span));
    };
    let mut args = vec![];
    loop {
        if let Some(next) = lex.peek()
            && next.token == Token::CloseBrackets
        {
            lex.next();
            break;
        }
        let arg = build_expression_with_priority(lex, 0, |f| {
            f == Some(&Token::CloseBrackets) || f == Some(&Token::Comma)
        })?;
        args.push(arg);
        if let Some(next) = lex.peek()
            && next.token == Token::Comma
        {
            lex.next();
        }
    }
    Ok(Expression::Function(Function::new(name, args)))
}
