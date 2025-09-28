use std::iter::Peekable;

use crate::parser::{
    ast::{expression::Expression, substr::Substring},
    expression::build_expression_with_priority,
    lexer::LexerItem,
    parser_error::ParserError,
    tokens::Token,
};

impl Substring {
    pub(crate) fn new(
        super_string: Expression,
        substring_from: Option<Expression>,
        substring_for: Option<Expression>,
    ) -> Self {
        Self {
            super_string: Box::new(super_string),
            substring_for: substring_for.map(Box::new),
            substring_from: substring_from.map(Box::new),
        }
    }
}

pub(super) fn build_substring(
    lex: &mut Peekable<impl Iterator<Item = LexerItem>>,
) -> Result<Expression, ParserError> {
    let Some(open) = lex.next() else {
        return Err(ParserError::UnexpectedEof);
    };
    if open.token != Token::OpenBrackets {
        return Err(ParserError::UnexpectedToken(open.span));
    };
    let super_string = build_expression_with_priority(lex, 0, |f| {
        f == Some(&Token::From) || f == Some(&Token::For)
    })?;
    if let Some(next) = lex.next()
        && next.token == Token::From
    {
        let substring_from = build_expression_with_priority(lex, 0, |f| {
            f == Some(&Token::CloseBrackets) || f == Some(&Token::For)
        })?;
        if let Some(next) = lex.next()
            && next.token == Token::CloseBrackets
        {
            Ok(Expression::Substring(Substring::new(
                super_string,
                Some(substring_from),
                None,
            )))
        } else {
            let substring_for =
                build_expression_with_priority(lex, 0, |f| f == Some(&Token::CloseBrackets))?;
            lex.next();
            Ok(Expression::Substring(Substring::new(
                super_string,
                Some(substring_from),
                Some(substring_for),
            )))
        }
    } else {
        let substring_for = build_expression_with_priority(lex, 0, |f| {
            f == Some(&Token::CloseBrackets) || f == Some(&Token::From)
        })?;
        if let Some(next) = lex.next()
            && next.token == Token::CloseBrackets
        {
            Ok(Expression::Substring(Substring::new(
                super_string,
                None,
                Some(substring_for),
            )))
        } else {
            let substring_from =
                build_expression_with_priority(lex, 0, |f| f == Some(&Token::CloseBrackets))?;
            lex.next();
            Ok(Expression::Substring(Substring::new(
                super_string,
                Some(substring_from),
                Some(substring_for),
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::parse_expression;

    #[test]
    fn test_substr_just_name() {
        let source = "substr";
        let err = parse_expression(source).err();

        assert!(err.is_some());
    }

    #[test]
    fn test_substr_with_no_open_brackets() {
        let source = "subString +";
        let err = parse_expression(source).err();

        assert!(err.is_some());
    }

    #[test]
    fn test_substr_with_no_for_no_from() {
        let source = "subString(name)";
        let err = parse_expression(source).err();

        assert!(err.is_some());
    }

    #[test]
    fn test_substr_with_nothing_after_name() {
        let source = "subString(name";
        let err = parse_expression(source).err();

        assert!(err.is_some());
    }

    #[test]
    fn test_substr_with_nothing_after_from() {
        let source = "subString(name from 1";
        let err = parse_expression(source).err();

        assert!(err.is_some());
    }

    #[test]
    fn test_substr_with_nothing_after_for() {
        let source = "subString(name for 10";
        let err = parse_expression(source).err();

        assert!(err.is_some());
    }

    #[test]
    fn test_substr_with_nothing_after_from_with_from() {
        let source = "subString(name from 1 for";
        let err = parse_expression(source).err();

        assert!(err.is_some());
    }
}
