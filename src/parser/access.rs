use std::iter::Peekable;

use crate::parser::{
    ast::{access::Access, expression::Expression},
    lexer::LexerItem,
    parser_error::ParserError,
    tokens::Token,
};

impl Access {
    pub(super) fn from_str(name: &str) -> Option<Self> {
        match name {
            "PARENT" => Some(Access::Parent),
            "NAME" => Some(Access::Name),
            "STEM" => Some(Access::Stem),
            "PATH" => Some(Access::Path),
            "EXTENSION" => Some(Access::Extension),
            "CONTENT" => Some(Access::Content),
            "DEPTH" => Some(Access::Depth),
            "SIZE" => Some(Access::Size),
            "COUNT" => Some(Access::Count),
            "CREATED" => Some(Access::Created),
            "MODIFIED" => Some(Access::Modified),
            "EXISTS" => Some(Access::Exists),
            "OWNER" => Some(Access::Owner),
            "GROUP" => Some(Access::Group),
            "PERMISSIONS" => Some(Access::Permissions),
            "ABSOLUTE" => Some(Access::Absolute),
            "FILES" => Some(Access::Files),
            "ME" | "SELF" | "THIS" => Some(Access::Me),
            _ => None,
        }
    }
}

pub(super) fn read_access(
    access: Access,
    lex: &mut Peekable<impl Iterator<Item = LexerItem>>,
) -> Result<Expression, ParserError> {
    if let Some(next) = lex.peek() {
        if next.token == Token::OpenBrackets {
            lex.next();
            if let Some(next) = lex.peek() {
                if next.token == Token::CloseBrackets {
                    lex.next();
                    Ok(Expression::Access(access))
                } else {
                    Err(ParserError::UnexpectedToken(next.span))
                }
            } else {
                Err(ParserError::UnexpectedEof)
            }
        } else {
            Ok(Expression::Access(access))
        }
    } else {
        Ok(Expression::Access(access))
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::{
        ast::{access::Access, expression::Expression},
        parse_expression,
        parser_error::ParserError,
    };

    #[test]
    fn test_access_without_brackets() -> Result<(), ParserError> {
        let source = "content";
        let expr = parse_expression(source)?;

        assert!(matches!(expr, Expression::Access(Access::Content)));

        Ok(())
    }

    #[test]
    fn test_access_with_brackets() -> Result<(), ParserError> {
        let source = "content()";
        let expr = parse_expression(source)?;

        assert!(matches!(expr, Expression::Access(Access::Content)));

        Ok(())
    }

    #[test]
    fn test_access_with_bracket_non_close() -> Result<(), ParserError> {
        let source = "content( ";
        let err = parse_expression(source).err();

        assert!(err.is_some());

        Ok(())
    }

    #[test]
    fn test_access_with_bracket_unexpected() -> Result<(), ParserError> {
        let source = "content(12)";
        let err = parse_expression(source).err();

        assert!(err.is_some());

        Ok(())
    }
}
