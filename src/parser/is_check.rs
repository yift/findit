use crate::{
    parser::{
        ast::{
            expression::Expression,
            is_check::{IsCheck, IsType},
        },
        lexer::LexerItem,
        parser_error::ParserError,
        tokens::Token,
    },
    value::Value,
};

impl IsCheck {
    pub(super) fn new(expression: Expression, check_type: IsType, negate: bool) -> Self {
        Self {
            expression: Box::new(expression),
            check_type,
            negate,
        }
    }
}

impl TryFrom<LexerItem> for IsType {
    type Error = ParserError;
    fn try_from(value: LexerItem) -> Result<Self, Self::Error> {
        match value.token {
            Token::Value(Value::Bool(true)) => Ok(IsType::True),
            Token::Value(Value::Bool(false)) => Ok(IsType::False),
            Token::Some => Ok(IsType::Some),
            Token::None => Ok(IsType::None),
            _ => Err(ParserError::UnexpectedToken(value.span)),
        }
    }
}
