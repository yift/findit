use std::iter::Peekable;

use crate::parser::{
    ast::{
        expression::Expression,
        replace::{Replace, ReplaceWhat},
    },
    expression::build_expression_with_priority,
    lexer::LexerItem,
    parser_error::ParserError,
    tokens::Token,
};

impl ReplaceWhat {
    pub(super) fn new_pattern(pattern: Expression) -> Self {
        Self::Pattern(Box::new(pattern))
    }
    pub(super) fn new_string(pattern: Expression) -> Self {
        Self::String(Box::new(pattern))
    }
}

impl Replace {
    pub(super) fn new(source: Expression, what: ReplaceWhat, to: Expression) -> Self {
        Self {
            source: Box::new(source),
            what,
            to: Box::new(to),
        }
    }
}

pub(super) fn build_replace(
    lex: &mut Peekable<impl Iterator<Item = LexerItem>>,
) -> Result<Expression, ParserError> {
    let Some(open) = lex.next() else {
        return Err(ParserError::UnexpectedEof);
    };
    if open.token != Token::OpenBrackets {
        return Err(ParserError::UnexpectedToken(open.span));
    }
    let source = build_expression_with_priority(lex, 0, |f| {
        f == Some(&Token::From) || f == Some(&Token::Pattern)
    })?;
    let regex = if let Some(next) = lex.next()
        && next.token == Token::Pattern
    {
        true
    } else {
        false
    };
    let what = build_expression_with_priority(lex, 0, |f| f == Some(&Token::To))?;
    let what = if regex {
        ReplaceWhat::new_pattern(what)
    } else {
        ReplaceWhat::new_string(what)
    };
    lex.next();

    let to = build_expression_with_priority(lex, 0, |f| f == Some(&Token::CloseBrackets))?;
    lex.next();
    Ok(Expression::Replace(Replace::new(source, what, to)))
}

#[cfg(test)]
mod tests {
    use crate::parser::parse_expression;

    #[test]
    fn test_replace_just_name() {
        let source = "replace";
        let err = parse_expression(source).err();

        assert!(err.is_some());
    }
    #[test]
    fn test_replace_with_no_open_brackets() {
        let source = "replace +";
        let err = parse_expression(source).err();

        assert!(err.is_some());
    }
}
