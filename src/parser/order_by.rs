use crate::parser::{
    expression::{Expression, ParserError, build_expression_with_priority},
    lexer::lex,
    tokens::Token,
};

#[derive(Debug, PartialEq)]
pub(crate) enum OrderByDirection {
    Asc,
    Desc,
}

#[derive(Debug, PartialEq)]
pub(crate) struct OrderByItem {
    pub(crate) expression: Expression,
    pub(crate) direction: OrderByDirection,
}

#[derive(Debug, PartialEq)]
pub(crate) struct OrderByExpression {
    pub(crate) items: Vec<OrderByItem>,
}

pub(crate) fn parse_order_by(source: &str) -> Result<OrderByExpression, ParserError> {
    let mut lexer = lex(source)?;

    let mut items = vec![];
    loop {
        let expression = build_expression_with_priority(&mut lexer, 0, |f| {
            f.is_none()
                || f == Some(&Token::Asc)
                || f == Some(&Token::Comma)
                || f == Some(&Token::Desc)
        })?;
        let mut next = lexer.next();
        let next_token = next.as_ref().map(|f| f.token.clone());
        let direction = if next_token == Some(Token::Desc) {
            OrderByDirection::Desc
        } else {
            OrderByDirection::Asc
        };
        items.push(OrderByItem {
            expression,
            direction,
        });
        if next_token == Some(Token::Asc) || next_token == Some(Token::Desc) {
            next = lexer.next();
        };
        let Some(comma) = &next else {
            break;
        };
        if comma.token != Token::Comma {
            return Err(ParserError::UnexpectedToken(comma.span));
        }
    }

    Ok(OrderByExpression { items })
}

#[cfg(test)]
mod tests {

    use crate::{
        parser::{
            access::Access,
            expression::{Expression, ParserError},
            order_by::{OrderByDirection, OrderByExpression, OrderByItem, parse_order_by},
        },
        value::Value,
    };

    #[test]
    fn test_order_by_single_item() -> Result<(), ParserError> {
        let source = "1";
        let ast = parse_order_by(source)?;

        assert_eq!(
            ast,
            OrderByExpression {
                items: vec![OrderByItem {
                    expression: Expression::Literal(Value::Number(1)),
                    direction: OrderByDirection::Asc,
                }]
            }
        );

        Ok(())
    }

    #[test]
    fn test_order_by_three_items() -> Result<(), ParserError> {
        let source = "1, false, name";
        let ast = parse_order_by(source)?;

        assert_eq!(
            ast,
            OrderByExpression {
                items: vec![
                    OrderByItem {
                        expression: Expression::Literal(Value::Number(1)),
                        direction: OrderByDirection::Asc,
                    },
                    OrderByItem {
                        expression: Expression::Literal(Value::Bool(false)),
                        direction: OrderByDirection::Asc,
                    },
                    OrderByItem {
                        expression: Expression::Access(Access::Name),
                        direction: OrderByDirection::Asc,
                    },
                ]
            }
        );

        Ok(())
    }

    #[test]
    fn test_order_by_three_items_asc_and_desc() -> Result<(), ParserError> {
        let source = "1 asc, false, name desc";
        let ast = parse_order_by(source)?;

        assert_eq!(
            ast,
            OrderByExpression {
                items: vec![
                    OrderByItem {
                        expression: Expression::Literal(Value::Number(1)),
                        direction: OrderByDirection::Asc,
                    },
                    OrderByItem {
                        expression: Expression::Literal(Value::Bool(false)),
                        direction: OrderByDirection::Asc,
                    },
                    OrderByItem {
                        expression: Expression::Access(Access::Name),
                        direction: OrderByDirection::Desc,
                    },
                ]
            }
        );

        Ok(())
    }

    #[test]
    fn test_order_by_no_comma() {
        let source = "10 20";
        let err = parse_order_by(source).err();

        assert!(err.is_some());
    }

    #[test]
    fn test_order_by_desc_no_comma() {
        let source = "10 desc 20";
        let err = parse_order_by(source).err();

        assert!(err.is_some());
    }
}
