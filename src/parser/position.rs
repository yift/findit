use std::iter::Peekable;

use crate::parser::{
    ast::{expression::Expression, position::Position},
    expression::build_expression_with_priority,
    lexer::LexerItem,
    parser_error::ParserError,
    tokens::Token,
};

impl Position {
    pub(crate) fn new(sub_string: Expression, super_string: Expression) -> Self {
        Self {
            sub_string: Box::new(sub_string),
            super_string: Box::new(super_string),
        }
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

#[cfg(test)]
mod tests {
    use crate::parser::parse_expression;

    #[test]
    fn test_position_just_name() {
        let source = "position";
        let err = parse_expression(source).err();

        assert!(err.is_some());
    }
    #[test]
    fn test_position_with_no_open_brackets() {
        let source = "position +";
        let err = parse_expression(source).err();

        assert!(err.is_some());
    }
}
