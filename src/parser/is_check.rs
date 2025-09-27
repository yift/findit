use crate::{
    parser::{
        expression::{Expression, ParserError},
        lexer::LexerItem,
        tokens::Token,
    },
    value::Value,
};

#[derive(Debug, PartialEq)]
pub(crate) struct IsCheck {
    pub(crate) expression: Box<Expression>,
    pub(crate) check_type: IsType,
    pub(crate) negate: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum IsType {
    True,
    False,
    None,
    Some,
}

impl IsCheck {
    pub(crate) fn new(expression: Expression, check_type: IsType, negate: bool) -> Self {
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
