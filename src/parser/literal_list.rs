use std::iter::Peekable;

use crate::parser::{
    ast::{expression::Expression, list::List},
    expression::build_expression_with_priority,
    lexer::LexerItem,
    parser_error::ParserError,
    tokens::Token,
};

impl List {
    pub(crate) fn new(items: Vec<Expression>) -> Self {
        Self { items }
    }
}

pub(super) fn build_literal_list(
    lex: &mut Peekable<impl Iterator<Item = LexerItem>>,
) -> Result<Expression, ParserError> {
    let mut items = vec![];
    loop {
        if let Some(next) = lex.peek()
            && next.token == Token::ListEnds
        {
            lex.next();
            break;
        }
        let arg = build_expression_with_priority(lex, 0, |f| {
            f == Some(&Token::ListEnds) || f == Some(&Token::Comma)
        })?;
        items.push(arg);
        if let Some(next) = lex.peek()
            && next.token == Token::Comma
        {
            lex.next();
        }
    }
    Ok(Expression::List(List::new(items)))
}

#[cfg(test)]
mod tests {
    use crate::{
        parser::{
            ast::{expression::Expression, list::List},
            parse_expression,
        },
        value::Value,
    };

    #[test]
    fn test_list_never_ends() {
        let source = "[10, 20";
        let err = parse_expression(source).err();

        assert!(err.is_some());
    }
    #[test]
    fn test_list_no_commas() {
        let source = "[10 20]";
        let err = parse_expression(source).err();

        assert!(err.is_some());
    }

    #[test]
    fn test_list_simple() {
        let source = "[10, 20]";
        let expr = parse_expression(source).unwrap();

        assert_eq!(
            expr,
            Expression::List(List::new(vec![
                Expression::Literal(Value::Number(10)),
                Expression::Literal(Value::Number(20))
            ]))
        );
    }
}
