use crate::parser::{
    ast::expression::Expression,
    ast::order_by::{OrderByDirection, OrderByExpression, OrderByItem},
    expression::build_expression_with_priority,
    lexer::lex,
    parser_error::ParserError,
    tokens::Token,
};

mod access;
pub(crate) mod ast;
mod between;
mod binary_expression;
mod case;
mod cast;
mod define_class;
mod execute;
mod expression;
mod format;
mod function;
mod function_name;
mod if_expression;
mod is_check;
mod lexer;
mod literal_list;
mod method;
mod negate;
mod order_by;
mod parse_date;
pub(crate) mod parser_error;
mod replace;
mod self_divide;
mod span;
mod tokens;
mod with;

pub(crate) fn parse_expression(source: &str) -> Result<Expression, ParserError> {
    let mut lexer = lex(source)?;

    build_expression_with_priority(&mut lexer, 0, |f| f.is_none())
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
