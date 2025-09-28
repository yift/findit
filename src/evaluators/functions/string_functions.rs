use std::collections::VecDeque;

use regex::Regex;

use crate::{
    errors::FindItError,
    evaluators::expr::{Evaluator, get_eval},
    file_wrapper::FileWrapper,
    parser::ast::position::Position as PositionExpression,
    parser::ast::substr::Substring,
    value::{Value, ValueType},
};

pub(crate) fn new_regex(
    expr: Box<dyn Evaluator>,
    pattern: Box<dyn Evaluator>,
) -> Result<Box<dyn Evaluator>, FindItError> {
    if expr.expected_type() != ValueType::String {
        return Err(FindItError::BadExpression(
            "REGULAR expressions can only work with strings".into(),
        ));
    }
    if pattern.expected_type() != ValueType::String {
        return Err(FindItError::BadExpression(
            "REGULAR expressions pattern can only be strings".into(),
        ));
    }
    Ok(Box::new(Regexp { expr, pattern }))
}

struct Regexp {
    expr: Box<dyn Evaluator>,
    pattern: Box<dyn Evaluator>,
}
impl Evaluator for Regexp {
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::String(expr) = self.expr.eval(file) else {
            return Value::Empty;
        };
        let Value::String(pattern) = self.pattern.eval(file) else {
            return Value::Empty;
        };
        let Ok(regexp) = Regex::new(&pattern) else {
            return Value::Empty;
        };
        regexp.is_match(&expr).into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Bool
    }
}

impl TryFrom<&PositionExpression> for Box<dyn Evaluator> {
    type Error = FindItError;
    fn try_from(position: &PositionExpression) -> Result<Self, Self::Error> {
        let str = get_eval(&position.super_string)?;
        let sub_str = get_eval(&position.sub_string)?;

        if (str.expected_type(), sub_str.expected_type()) != (ValueType::String, ValueType::String)
        {
            return Err(FindItError::BadExpression(
                "POSITION can only work with strings".into(),
            ));
        }
        Ok(Box::new(Position { str, sub_str }))
    }
}

struct Position {
    str: Box<dyn Evaluator>,
    sub_str: Box<dyn Evaluator>,
}
impl Evaluator for Position {
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::String(str) = self.str.eval(file) else {
            return Value::Empty;
        };
        let Value::String(sub_str) = self.sub_str.eval(file) else {
            return Value::Empty;
        };
        str.find(&sub_str).map(|i| i + 1).unwrap_or_default().into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Number
    }
}

impl TryFrom<&Substring> for Box<dyn Evaluator> {
    type Error = FindItError;
    fn try_from(substr: &Substring) -> Result<Self, Self::Error> {
        let str = get_eval(&substr.super_string)?;
        if str.expected_type() != ValueType::String {
            return Err(FindItError::BadExpression(
                "SUBSTRING can only work with strings".into(),
            ));
        }
        let from = if let Some(from) = &substr.substring_from {
            let from = get_eval(from)?;
            if from.expected_type() != ValueType::Number {
                return Err(FindItError::BadExpression(
                    "SUBSTRING can only start from a number".into(),
                ));
            }
            Some(from)
        } else {
            None
        };
        let length = if let Some(length) = &substr.substring_for {
            let length = get_eval(length)?;
            if length.expected_type() != ValueType::Number {
                return Err(FindItError::BadExpression(
                    "SUBSTRING can only have numeric length".into(),
                ));
            }
            Some(length)
        } else {
            None
        };
        Ok(Box::new(SubString { str, from, length }))
    }
}

struct SubString {
    str: Box<dyn Evaluator>,
    from: Option<Box<dyn Evaluator>>,
    length: Option<Box<dyn Evaluator>>,
}
impl Evaluator for SubString {
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::String(str) = self.str.eval(file) else {
            return Value::Empty;
        };
        let mut str = str.as_str();

        if let Some(from) = &self.from {
            let Value::Number(from) = from.eval(file) else {
                return Value::Empty;
            };
            let Ok(from) = usize::try_from(from) else {
                return Value::Empty;
            };
            if from > str.len() {
                return "".into();
            }
            str = &str[from..];
        }
        if let Some(length) = &self.length {
            let Value::Number(length) = length.eval(file) else {
                return Value::Empty;
            };
            let Ok(length) = length.try_into() else {
                return Value::Empty;
            };
            if length >= str.len() {
                return str.into();
            }
            str = &str[..length];
        }

        str.into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::String
    }
}

pub(crate) enum TrimWhere {
    Head,
    Tail,
    Both,
}
pub(crate) fn new_trim(
    mut args: VecDeque<Box<dyn Evaluator>>,
    trim_where: TrimWhere,
) -> Result<Box<dyn Evaluator>, FindItError> {
    if args.len() > 1 {
        return Err(FindItError::BadExpression(
            "TRIM mut have only one argument".into(),
        ));
    }
    let Some(str) = args.pop_front() else {
        return Err(FindItError::BadExpression(
            "TRIM mut have one argument".into(),
        ));
    };
    if str.expected_type() != ValueType::String {
        return Err(FindItError::BadExpression(
            "TRIM can only work with strings".into(),
        ));
    }
    Ok(Box::new(Trim { str, trim_where }))
}

struct Trim {
    str: Box<dyn Evaluator>,
    trim_where: TrimWhere,
}
impl Evaluator for Trim {
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::String(str) = self.str.eval(file) else {
            return Value::Empty;
        };
        match self.trim_where {
            TrimWhere::Head => str.trim_start(),
            TrimWhere::Tail => str.trim_end(),
            TrimWhere::Both => str.trim(),
        }
        .into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::String
    }
}

#[cfg(test)]
mod tests {

    use std::path::Path;

    use crate::{
        evaluators::expr::read_expr,
        file_wrapper::FileWrapper,
        value::{Value, ValueType},
    };

    #[test]
    fn regex_no_string_expr() {
        let err = read_expr("1 MATCHES \"a\"").err();
        assert!(err.is_some())
    }

    #[test]
    fn regex_no_string_pattern() {
        let err = read_expr("\"a\" matches 1").err();
        assert!(err.is_some())
    }

    #[test]
    fn regex_null_expr_return_empty() {
        let eval = read_expr("content MATCHES \"abc\"").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn regex_null_pattern_return_empty() {
        let eval = read_expr("\"abc\" MATCHES content").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn regex_bad_pattern_return_empty() {
        let eval = read_expr("\"abc\" MATCHES \"[\"").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn position_no_string_expr() {
        let err = read_expr("POSITION(\"txt\" IN 12)").err();
        assert!(err.is_some())
    }

    #[test]
    fn position_no_string_str() {
        let err = read_expr("POSITION(12 IN path)").err();
        assert!(err.is_some())
    }

    #[test]
    fn position_expect_number() {
        let expr = read_expr("POSITION(\"a\" IN path)").unwrap();
        assert_eq!(expr.expected_type(), ValueType::Number);
    }

    #[test]
    fn position_null_expr_return_empty() {
        let eval = read_expr("POSITION(content IN \"abc\")").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn position_null_str_return_empty() {
        let eval = read_expr("POSITION(\"abc\" IN content)").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn substring_no_from_not_for_return_error() {
        let err = read_expr("SUBSTRING(content)").err();
        assert!(err.is_some())
    }

    #[test]
    fn substring_from_not_a_number() {
        let err = read_expr("SUBSTRING(content FROM path)").err();
        assert!(err.is_some())
    }

    #[test]
    fn substring_for_not_a_number() {
        let err = read_expr("SUBSTRING(content FOR path)").err();
        assert!(err.is_some())
    }

    #[test]
    fn substring_expr_not_a_string() {
        let err = read_expr("SUBSTRING(12 for 12)").err();
        assert!(err.is_some())
    }

    #[test]
    fn substring_expect_string() {
        let expr = read_expr("SUBSTRING(content FROM 1 FOR 4)").unwrap();
        assert_eq!(expr.expected_type(), ValueType::String);
    }

    #[test]
    fn substring_null_for_return_empty() {
        let eval = read_expr("SUBSTRING(\"abc\" FOR length)").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn substring_null_from_return_empty() {
        let eval = read_expr("SUBSTRING(\"abc\" FROM length)").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn substring_null_str_return_empty() {
        let eval = read_expr("SUBSTRING(content FROM 2)").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn trim_no_string_expr() {
        let err = read_expr("TRIM(12)").err();
        assert!(err.is_some())
    }

    #[test]
    fn trim_no_args() {
        let err = read_expr("TRIM()").err();
        assert!(err.is_some())
    }

    #[test]
    fn trim_too_many_args() {
        let err = read_expr("TRIM(\"abc\", \"def\")").err();
        assert!(err.is_some())
    }

    #[test]
    fn trim_null_str_return_empty() {
        let eval = read_expr("TRIM(content)").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn trim_expect_string() {
        let expr = read_expr("TRIM(\"\")").unwrap();
        assert_eq!(expr.expected_type(), ValueType::String);
    }
}
