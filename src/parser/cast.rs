use crate::parser::{
    ast::{
        access::Access,
        as_cast::{As, CastType},
        expression::Expression,
    },
    lexer::LexerItem,
    parser_error::ParserError,
    tokens::Token,
};

impl As {
    pub(super) fn new(expression: Expression, cast_type: CastType) -> Self {
        Self {
            expression: Box::new(expression),
            cast_type,
        }
    }
}

impl TryFrom<LexerItem> for CastType {
    type Error = ParserError;
    fn try_from(value: LexerItem) -> Result<Self, Self::Error> {
        match value.token {
            Token::Dir | Token::File | Token::SimpleAccess(Access::Path) => Ok(CastType::Path),
            Token::Boolean => Ok(CastType::Bool),
            Token::Number => Ok(CastType::Number),
            Token::Date => Ok(CastType::Date),
            Token::String => Ok(CastType::String),
            _ => Err(ParserError::UnexpectedToken(value.span)),
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::parser::parse_expression;

    #[test]
    fn parse_without_type() {
        let src = "self as";
        let err = parse_expression(src).err();

        assert!(err.is_some())
    }

    #[test]
    fn parse_with_bad_type() {
        let src = "self as 12";
        let err = parse_expression(src).err();

        assert!(err.is_some())
    }
}
