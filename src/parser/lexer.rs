use std::{error::Error, fmt::Display, iter::Peekable};

use crate::parser::{span::Span, tokens::Token};

#[derive(Debug)]
pub struct LexerError {
    cause: String,
    span: Span,
}
impl Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error: {} at: {}", self.cause, self.span)
    }
}
impl Error for LexerError {}

#[derive(Debug, PartialEq, Clone)]
pub(super) struct LexerItem {
    pub(crate) token: Token,
    pub(crate) span: Span,
}

pub(super) fn lex(
    expression: &str,
) -> Result<Peekable<impl Iterator<Item = LexerItem>>, LexerError> {
    let mut items: Vec<LexerItem> = vec![];
    let mut chars = expression.chars().enumerate().peekable();
    let mut start = 0;
    loop {
        let token = Token::new(&mut chars);
        let end = match chars.peek() {
            Some((end, _)) => *end,
            _ => expression.len(),
        };
        let span = Span { start, end };
        let token = match token {
            Ok(token) => token,
            Err(err) => {
                return Err(LexerError {
                    cause: err.cause,
                    span,
                });
            }
        };
        let Some(token) = token else {
            break;
        };
        items.push(LexerItem { token, span });
        start = end;
    }
    Ok(items.into_iter().peekable())
}

#[cfg(test)]
mod tests {
    use crate::{
        parser::operator::{ArithmeticOperator, BinaryOperator},
        value::Value,
    };

    use super::*;

    #[test]
    fn new_simple_expr() -> Result<(), LexerError> {
        let str = "10 \t+   321";

        let lexer: Vec<_> = lex(str)?.collect();

        assert_eq!(lexer.len(), 3);
        assert_eq!(
            lexer[0],
            LexerItem {
                span: Span { start: 0, end: 2 },
                token: Token::Value(Value::Number(10)),
            }
        );
        assert_eq!(
            lexer[1],
            LexerItem {
                span: Span { start: 2, end: 5 },
                token: Token::BinaryOperator(BinaryOperator::Arithmetic(ArithmeticOperator::Plus)),
            }
        );
        assert_eq!(
            lexer[2],
            LexerItem {
                span: Span { start: 5, end: 11 },
                token: Token::Value(Value::Number(321)),
            }
        );

        Ok(())
    }

    #[test]
    fn new_with_err() {
        let err = lex("10 + } - 2").err();

        let span = err.map(|f| f.span);

        assert_eq!(span, Some(Span { start: 4, end: 5 }));
    }
}
