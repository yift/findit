use crate::parser::{
    expression::{Expression, ParserError, build_expression_with_priority},
    lexer::lex,
    tokens::Token,
};

pub(crate) enum OrderByDirection {
    Asc,
    Desc,
}

pub(crate) struct OrderByItem {
    pub(crate) expression: Expression,
    pub(crate) direction: OrderByDirection,
}

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
