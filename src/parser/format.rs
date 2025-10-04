use std::iter::Peekable;

use crate::parser::{
    ast::{expression::Expression, format::Format},
    expression::build_expression_with_priority,
    lexer::LexerItem,
    parser_error::ParserError,
    tokens::Token,
};

impl Format {
    pub(super) fn new(timestamp: Expression, format: Expression) -> Self {
        Self {
            timestamp: Box::new(timestamp),
            format: Box::new(format),
        }
    }
}

pub(super) fn build_format(
    lex: &mut Peekable<impl Iterator<Item = LexerItem>>,
) -> Result<Expression, ParserError> {
    let Some(open) = lex.next() else {
        return Err(ParserError::UnexpectedEof);
    };
    if open.token != Token::OpenBrackets {
        return Err(ParserError::UnexpectedToken(open.span));
    }
    let timestamp = build_expression_with_priority(lex, 0, |f| f == Some(&Token::As))?;
    lex.next();
    let format = build_expression_with_priority(lex, 0, |f| f == Some(&Token::CloseBrackets))?;
    lex.next();
    Ok(Expression::Format(Format::new(timestamp, format)))
}

#[cfg(test)]
mod tests {
    use crate::parser::parse_expression;

    #[test]
    fn test_format_just_name() {
        let source = "format";
        let err = parse_expression(source).err();

        assert!(err.is_some());
    }
    #[test]
    fn test_format_with_no_open_brackets() {
        let source = "format +";
        let err = parse_expression(source).err();

        assert!(err.is_some());
    }
}
