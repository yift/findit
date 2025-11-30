use std::iter::Peekable;

use crate::parser::{
    ast::{
        class::{ClassDefinition, Field},
        expression::Expression,
    },
    expression::build_expression_with_priority,
    lexer::LexerItem,
    parser_error::ParserError,
    tokens::Token,
};

pub(super) fn build_class_definition(
    lex: &mut Peekable<impl Iterator<Item = LexerItem>>,
) -> Result<Expression, ParserError> {
    let mut fields = vec![];
    loop {
        if let Some(next) = lex.peek()
            && next.token == Token::ClassEnds
        {
            lex.next();
            break;
        }

        let Some(name) = lex.next() else {
            return Err(ParserError::UnexpectedEof);
        };
        let LexerItem {
            token: Token::ClassFieldName(name),
            span: _,
        } = name
        else {
            return Err(ParserError::UnexpectedToken(name.span));
        };
        let value = build_expression_with_priority(lex, 0, |f| {
            f == Some(&Token::ClassEnds) || f == Some(&Token::Comma)
        })?;
        let field = Field { name, value };
        fields.push(field);
        if let Some(next) = lex.peek()
            && next.token == Token::Comma
        {
            lex.next();
        }
    }
    Ok(Expression::ClassDefinition(ClassDefinition { fields }))
}

#[cfg(test)]
mod tests {
    use crate::parser::parse_expression;

    #[test]
    fn parse_without_end() {
        let src = "{:name 1";
        let err = parse_expression(src).err();

        assert!(err.is_some())
    }

    #[test]
    fn parse_without_value() {
        let src = "{:name :key}";
        let err = parse_expression(src).err();

        assert!(err.is_some())
    }

    #[test]
    fn parse_without_comma() {
        let src = "{:name 1 :f2 2}";
        let err = parse_expression(src).err();

        assert!(err.is_some())
    }

    #[test]
    fn parse_without_name() {
        let src = "{";
        let err = parse_expression(src).err();

        assert!(err.is_some())
    }

    #[test]
    fn parse_wit_not_a_name() {
        let src = "{1";
        let err = parse_expression(src).err();

        assert!(err.is_some())
    }
}
