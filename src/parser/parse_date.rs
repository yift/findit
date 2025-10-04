use std::iter::Peekable;

use crate::parser::{
    ast::{expression::Expression, parse::Parse},
    expression::build_expression_with_priority,
    lexer::LexerItem,
    parser_error::ParserError,
    tokens::Token,
};

impl Parse {
    pub(super) fn new(str: Expression, format: Expression) -> Self {
        Self {
            str: Box::new(str),
            format: Box::new(format),
        }
    }
}

pub(super) fn build_parse_date(
    lex: &mut Peekable<impl Iterator<Item = LexerItem>>,
) -> Result<Expression, ParserError> {
    let Some(open) = lex.next() else {
        return Err(ParserError::UnexpectedEof);
    };
    if open.token != Token::OpenBrackets {
        return Err(ParserError::UnexpectedToken(open.span));
    }
    let str = build_expression_with_priority(lex, 0, |f| f == Some(&Token::From))?;
    lex.next();
    let format = build_expression_with_priority(lex, 0, |f| f == Some(&Token::CloseBrackets))?;
    lex.next();
    Ok(Expression::Parse(Parse::new(str, format)))
}

#[cfg(test)]
mod tests {
    use crate::parser::parse_expression;

    #[test]
    fn test_pars_just_name() {
        let source = "parse";
        let err = parse_expression(source).err();

        assert!(err.is_some());
    }
    #[test]
    fn test_parser_with_no_open_brackets() {
        let source = "parse +";
        let err = parse_expression(source).err();

        assert!(err.is_some());
    }
}
