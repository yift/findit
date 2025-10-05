use std::iter::Peekable;

use crate::parser::{
    ast::{expression::Expression, with::With},
    expression::build_expression_with_priority,
    lexer::LexerItem,
    parser_error::ParserError,
    tokens::Token,
};

impl With {
    fn new(names: Vec<(String, Expression)>, action: Expression) -> Self {
        Self {
            names: names
                .into_iter()
                .map(|(name, expr)| (name, Box::new(expr)))
                .collect(),
            action: Box::new(action),
        }
    }
}
pub(super) fn build_with(
    lex: &mut Peekable<impl Iterator<Item = LexerItem>>,
) -> Result<Expression, ParserError> {
    let mut names = vec![];
    loop {
        let Some(name) = lex.next() else {
            return Err(ParserError::UnexpectedEof);
        };
        let Token::BindingName(name) = name.token else {
            return Err(ParserError::UnexpectedToken(name.span));
        };
        if let Some(next) = lex.peek()
            && next.token == Token::As
        {
            lex.next();
        };
        let expression = build_expression_with_priority(lex, 0, |f| {
            f == Some(&Token::Do) || f == Some(&Token::Comma)
        })?;
        names.push((name, expression));
        if let Some(next) = lex.next()
            && next.token == Token::Do
        {
            break;
        }
    }

    let action = build_expression_with_priority(lex, 0, |f| f == Some(&Token::End))?;
    lex.next();
    Ok(Expression::With(With::new(names, action)))
}

#[cfg(test)]
mod tests {
    use crate::parser::parse_expression;

    #[test]
    fn test_empty_with() {
        let source = "with";
        let err = parse_expression(source).err();

        assert!(err.is_some());
    }

    #[test]
    fn test_with_without_name() {
        let source = "with()";
        let err = parse_expression(source).err();

        assert!(err.is_some());
    }

    #[test]
    fn test_with_without_definitions() {
        let source = "with do 10 end";
        let err = parse_expression(source).err();

        assert!(err.is_some());
    }
}
