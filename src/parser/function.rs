use std::iter::Peekable;

use crate::parser::{
    ast::function_name::FunctionName,
    ast::{expression::Expression, function::Function},
    expression::build_expression_with_priority,
    lexer::LexerItem,
    parser_error::ParserError,
    tokens::Token,
};

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

#[cfg(test)]
mod tests {
    use crate::parser::parse_expression;

    #[test]
    fn test_func_just_name() {
        let source = "rand";
        let err = parse_expression(source).err();

        assert!(err.is_some());
    }

    #[test]
    fn test_func_with_no_open_brackets() {
        let source = "rand +";
        let err = parse_expression(source).err();

        assert!(err.is_some());
    }
}
