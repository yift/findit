use crate::parser::{
    ast::{
        expression::Expression,
        methods::{LambdaFunction, Method},
    },
    expression::build_expression_with_priority,
    lexer::LexerItem,
    parser_error::ParserError,
    tokens::Token,
};
use std::iter::Peekable;

#[derive(Debug, PartialEq, Clone, Copy)]
pub(super) enum MethodName {
    Length,
    ToUpper,
    ToLower,
    Trim,
    TrimHead,
    TrimTail,
    Reverse,
    Map,
    Filter,
    Sum,
    Sort,
    SortBy,
}
impl MethodName {
    pub(super) fn from_str(name: &str) -> Option<Self> {
        match name {
            "LENGTH" | "LEN" | "COUNT" | "SIZE" => Some(MethodName::Length),
            "TO_UPPER" | "UPPER" | "UPPERCASE" | "TOUPPER" => Some(MethodName::ToUpper),
            "TO_LOWER" | "LOWER" | "LOWERCASE" | "TOLOWER" => Some(MethodName::ToLower),
            "TRIM" => Some(MethodName::Trim),
            "TRIM_HEAD" | "TRIMHEAD" => Some(MethodName::TrimHead),
            "TRIM_TAIL" | "TRIMTAIL" => Some(MethodName::TrimTail),
            "REVERSE" => Some(MethodName::Reverse),
            "MAP" => Some(MethodName::Map),
            "FILTER" => Some(MethodName::Filter),
            "SUM" => Some(MethodName::Sum),
            "SORT" | "ORDER" => Some(MethodName::Sort),
            "SORT_BY" | "ORDER_BY" | "SORTBY" | "ORDERBY" => Some(MethodName::SortBy),
            _ => None,
        }
    }
}

impl LambdaFunction {
    fn new(parameter: String, body: Expression) -> Self {
        Self {
            parameter,
            body: Box::new(body),
        }
    }
}
pub(super) fn build_lambda(
    lex: &mut Peekable<impl Iterator<Item = LexerItem>>,
) -> Result<LambdaFunction, ParserError> {
    let Some(param) = lex.next() else {
        return Err(ParserError::UnexpectedEof);
    };
    let Token::BindingName(name) = param.token else {
        return Err(ParserError::UnexpectedToken(param.span));
    };
    let body = build_expression_with_priority(lex, 0, |f| f == Some(&Token::CloseBrackets))?;
    Ok(LambdaFunction::new(name, body))
}

pub(super) fn build_method(
    name: &MethodName,
    lex: &mut Peekable<impl Iterator<Item = LexerItem>>,
) -> Result<Method, ParserError> {
    let Some(open) = lex.next() else {
        return Err(ParserError::UnexpectedEof);
    };
    if open.token != Token::OpenBrackets {
        return Err(ParserError::UnexpectedToken(open.span));
    }
    let method = match name {
        MethodName::Length => Ok(Method::Length),
        MethodName::ToUpper => Ok(Method::ToUpper),
        MethodName::ToLower => Ok(Method::ToLower),
        MethodName::Trim => Ok(Method::Trim),
        MethodName::TrimHead => Ok(Method::TrimHead),
        MethodName::TrimTail => Ok(Method::TrimTail),
        MethodName::Reverse => Ok(Method::Reverse),
        MethodName::Map => {
            let lambda = build_lambda(lex)?;
            Ok(Method::Map(lambda))
        }
        MethodName::Filter => {
            let lambda = build_lambda(lex)?;
            Ok(Method::Filter(lambda))
        }
        MethodName::Sum => Ok(Method::Sum),
        MethodName::Sort => Ok(Method::Sort),
        MethodName::SortBy => {
            let lambda = build_lambda(lex)?;
            Ok(Method::SortBy(lambda))
        }
    };
    let Some(close) = lex.next() else {
        return Err(ParserError::UnexpectedEof);
    };
    if close.token != Token::CloseBrackets {
        return Err(ParserError::UnexpectedToken(close.span));
    }
    method
}

#[cfg(test)]
mod tests {
    use crate::parser::parse_expression;

    #[test]
    fn test_method_just_name() {
        let source = "trim";
        let err = parse_expression(source).err();

        assert!(err.is_some());
    }

    #[test]
    fn test_method_no_open_brackets() {
        let source = "trim +";
        let err = parse_expression(source).err();

        assert!(err.is_some());
    }

    #[test]
    fn test_method_no_close_brackets() {
        let source = "trim(";
        let err = parse_expression(source).err();

        assert!(err.is_some());
    }

    #[test]
    fn test_method_no_lambda() {
        let source = "map(";
        let err = parse_expression(source).err();

        assert!(err.is_some());
    }
    #[test]
    fn test_method_lambda_no_name() {
        let source = "map( +";
        let err = parse_expression(source).err();

        assert!(err.is_some());
    }
}
