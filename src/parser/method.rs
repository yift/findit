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
    Max,
    Min,
    Avg,
    Sort,
    SortBy,
    Distinct,
    DistinctBy,
    Skip,
    Take,
    Join,
    Split,
    Lines,
    Words,
    First,
    Last,
    Contains,
    IndexOf,
    FlatMap,
    All,
    Any,
    GroupBy,
    Enumerate,
    Walk,
    HasPrefix,
    HasSuffix,
    RemovePrefix,
    RemoveSuffix,
    Debug,
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
            "MAX" | "MAXIMUM" => Some(MethodName::Max),
            "MIN" | "MINIMUM" => Some(MethodName::Min),
            "AVG" | "AVERAGE" => Some(MethodName::Avg),
            "SORT" | "ORDER" => Some(MethodName::Sort),
            "SORT_BY" | "ORDER_BY" | "SORTBY" | "ORDERBY" => Some(MethodName::SortBy),
            "SKIP" => Some(MethodName::Skip),
            "TAKE" => Some(MethodName::Take),
            "JOIN" => Some(MethodName::Join),
            "SPLIT" => Some(MethodName::Split),
            "LINES" => Some(MethodName::Lines),
            "WORDS" => Some(MethodName::Words),
            "FIRST" => Some(MethodName::First),
            "LAST" => Some(MethodName::Last),
            "CONTAINS" => Some(MethodName::Contains),
            "INDEXOF" | "INDEX_OF" => Some(MethodName::IndexOf),
            "FLATMAP" | "FLAT_MAP" => Some(MethodName::FlatMap),
            "ALL" => Some(MethodName::All),
            "ANY" => Some(MethodName::Any),
            "DISTINCT" | "UNIQUE" => Some(MethodName::Distinct),
            "DISTINCT_BY" | "DISTINCTBY" | "UNIQUE_BY" | "UNIQUEBY" => Some(MethodName::DistinctBy),
            "GROUPBY" | "GROUP_BY" => Some(MethodName::GroupBy),
            "ENUMERATE" => Some(MethodName::Enumerate),
            "WALK" => Some(MethodName::Walk),
            "HAS_PREFIX" | "HASPREFIX" | "STARTS_WITH" | "STARTSWITH" => {
                Some(MethodName::HasPrefix)
            }
            "HAS_SUFFIX" | "HASSUFFIX" | "ENDS_WITH" | "ENDSWITH" => Some(MethodName::HasSuffix),
            "REMOVE_PREFIX" | "REMOVEPREFIX" => Some(MethodName::RemovePrefix),
            "REMOVE_SUFFIX" | "REMOVESUFFIX" => Some(MethodName::RemoveSuffix),
            "DEBUG" | "DBG" => Some(MethodName::Debug),
            _ => None,
        }
    }

    fn must_have_arguments(&self) -> bool {
        match self {
            MethodName::Length => false,
            MethodName::ToUpper => false,
            MethodName::ToLower => false,
            MethodName::Trim => false,
            MethodName::TrimHead => false,
            MethodName::TrimTail => false,
            MethodName::Reverse => false,
            MethodName::Map => true,
            MethodName::Filter => true,
            MethodName::Sum => false,
            MethodName::Max => false,
            MethodName::Min => false,
            MethodName::Avg => false,
            MethodName::Sort => false,
            MethodName::SortBy => true,
            MethodName::Distinct => false,
            MethodName::DistinctBy => true,
            MethodName::Skip => true,
            MethodName::Take => true,
            MethodName::Join => true,
            MethodName::Split => true,
            MethodName::HasPrefix => true,
            MethodName::HasSuffix => true,
            MethodName::RemovePrefix => true,
            MethodName::RemoveSuffix => true,
            MethodName::Lines => false,
            MethodName::Words => false,
            MethodName::First => false,
            MethodName::Last => false,
            MethodName::Contains => true,
            MethodName::IndexOf => true,
            MethodName::FlatMap => true,
            MethodName::All => true,
            MethodName::Any => true,
            MethodName::GroupBy => true,
            MethodName::Enumerate => false,
            MethodName::Walk => false,
            MethodName::Debug => true,
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
    let must_have_arguments = name.must_have_arguments();
    let open = if let Some(next) = lex.peek() {
        if next.token == Token::OpenBrackets {
            lex.next();
            true
        } else {
            if must_have_arguments {
                return Err(ParserError::UnexpectedToken(next.span));
            }
            false
        }
    } else {
        if must_have_arguments {
            return Err(ParserError::UnexpectedEof);
        }
        false
    };
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
        MethodName::Max => Ok(Method::Max),
        MethodName::Min => Ok(Method::Min),
        MethodName::Avg => Ok(Method::Avg),
        MethodName::Sort => Ok(Method::Sort),
        MethodName::SortBy => {
            let lambda = build_lambda(lex)?;
            Ok(Method::SortBy(lambda))
        }
        MethodName::Distinct => Ok(Method::Distinct),
        MethodName::DistinctBy => {
            let lambda = build_lambda(lex)?;
            Ok(Method::DistinctBy(lambda))
        }
        MethodName::Skip => {
            let expr =
                build_expression_with_priority(lex, 0, |f| f == Some(&Token::CloseBrackets))?;
            Ok(Method::Skip(Box::new(expr)))
        }
        MethodName::Take => {
            let expr =
                build_expression_with_priority(lex, 0, |f| f == Some(&Token::CloseBrackets))?;
            Ok(Method::Take(Box::new(expr)))
        }
        MethodName::Join => {
            let next = lex.peek();
            if let Some(LexerItem {
                token: Token::CloseBrackets,
                ..
            }) = next
            {
                Ok(Method::Join(None))
            } else {
                let expr =
                    build_expression_with_priority(lex, 0, |f| f == Some(&Token::CloseBrackets))?;
                Ok(Method::Join(Some(Box::new(expr))))
            }
        }
        MethodName::Split => {
            let expr =
                build_expression_with_priority(lex, 0, |f| f == Some(&Token::CloseBrackets))?;
            Ok(Method::Split(Box::new(expr)))
        }
        MethodName::HasPrefix => {
            let expr =
                build_expression_with_priority(lex, 0, |f| f == Some(&Token::CloseBrackets))?;
            Ok(Method::HasPrefix(Box::new(expr)))
        }
        MethodName::HasSuffix => {
            let expr =
                build_expression_with_priority(lex, 0, |f| f == Some(&Token::CloseBrackets))?;
            Ok(Method::HasSuffix(Box::new(expr)))
        }
        MethodName::RemovePrefix => {
            let expr =
                build_expression_with_priority(lex, 0, |f| f == Some(&Token::CloseBrackets))?;
            Ok(Method::RemovePrefix(Box::new(expr)))
        }
        MethodName::RemoveSuffix => {
            let expr =
                build_expression_with_priority(lex, 0, |f| f == Some(&Token::CloseBrackets))?;
            Ok(Method::RemoveSuffix(Box::new(expr)))
        }
        MethodName::Lines => Ok(Method::Lines),
        MethodName::Words => Ok(Method::Words),
        MethodName::First => Ok(Method::First),
        MethodName::Last => Ok(Method::Last),
        MethodName::Contains => {
            let expr =
                build_expression_with_priority(lex, 0, |f| f == Some(&Token::CloseBrackets))?;
            Ok(Method::Contains(Box::new(expr)))
        }
        MethodName::IndexOf => {
            let expr =
                build_expression_with_priority(lex, 0, |f| f == Some(&Token::CloseBrackets))?;
            Ok(Method::IndexOf(Box::new(expr)))
        }
        MethodName::FlatMap => {
            let lambda = build_lambda(lex)?;
            Ok(Method::FlatMap(lambda))
        }
        MethodName::All => {
            let lambda = build_lambda(lex)?;
            Ok(Method::All(lambda))
        }
        MethodName::Any => {
            let lambda = build_lambda(lex)?;
            Ok(Method::Any(lambda))
        }
        MethodName::GroupBy => {
            let lambda = build_lambda(lex)?;
            Ok(Method::GroupBy(lambda))
        }
        MethodName::Enumerate => Ok(Method::Enumerate),
        MethodName::Walk => Ok(Method::Walk),
        MethodName::Debug => {
            let lambda = build_lambda(lex)?;
            Ok(Method::Debug(lambda))
        }
    };
    if open {
        let Some(close) = lex.next() else {
            return Err(ParserError::UnexpectedEof);
        };
        if close.token != Token::CloseBrackets {
            return Err(ParserError::UnexpectedToken(close.span));
        }
    }
    method
}

#[cfg(test)]
mod tests {
    use crate::parser::parse_expression;

    #[test]
    fn test_method_just_name() {
        let source = "map";
        let err = parse_expression(source).err();

        assert!(err.is_some());
    }

    #[test]
    fn test_method_no_open_brackets() {
        let source = "filter +";
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
