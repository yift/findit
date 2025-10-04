use std::collections::VecDeque;

use regex::Regex;

use crate::{
    errors::FindItError,
    evaluators::expr::{Evaluator, get_eval},
    file_wrapper::FileWrapper,
    parser::ast::{
        position::Position as PositionExpression,
        replace::{Replace, ReplaceWhat},
        substr::Substring,
    },
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

struct ReplaceString {
    source: Box<dyn Evaluator>,
    from: Box<dyn Evaluator>,
    to: Box<dyn Evaluator>,
}
impl Evaluator for ReplaceString {
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::String(source) = self.source.eval(file) else {
            return Value::Empty;
        };
        let Value::String(from) = self.from.eval(file) else {
            return Value::Empty;
        };
        let Value::String(to) = self.to.eval(file) else {
            return Value::Empty;
        };
        source.as_str().replace(&from, &to).into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::String
    }
}

struct ReplaceRegex {
    source: Box<dyn Evaluator>,
    pattern: Box<dyn Evaluator>,
    to: Box<dyn Evaluator>,
}
impl Evaluator for ReplaceRegex {
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::String(source) = self.source.eval(file) else {
            return Value::Empty;
        };
        let Value::String(pattern) = self.pattern.eval(file) else {
            return Value::Empty;
        };
        let Ok(regexp) = Regex::new(&pattern) else {
            return Value::Empty;
        };
        let Value::String(to) = self.to.eval(file) else {
            return Value::Empty;
        };
        regexp.replace_all(&source, to).to_string().into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::String
    }
}

impl TryFrom<&Replace> for Box<dyn Evaluator> {
    type Error = FindItError;
    fn try_from(replace: &Replace) -> Result<Self, Self::Error> {
        let source = get_eval(&replace.source)?;
        let (what, regex) = match &replace.what {
            ReplaceWhat::Pattern(p) => (get_eval(p)?, true),
            ReplaceWhat::String(p) => (get_eval(p)?, false),
        };
        let to = get_eval(&replace.to)?;

        if (
            to.expected_type(),
            source.expected_type(),
            what.expected_type(),
        ) != (ValueType::String, ValueType::String, ValueType::String)
        {
            return Err(FindItError::BadExpression(
                "Replace can only work with strings".into(),
            ));
        }
        if regex {
            Ok(Box::new(ReplaceRegex {
                source,
                pattern: what,
                to,
            }))
        } else {
            Ok(Box::new(ReplaceString {
                source,
                from: what,
                to,
            }))
        }
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
            "TRIM must have only one argument".into(),
        ));
    }
    let Some(str) = args.pop_front() else {
        return Err(FindItError::BadExpression(
            "TRIM must have one argument".into(),
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

pub(crate) fn new_length(
    mut args: VecDeque<Box<dyn Evaluator>>,
) -> Result<Box<dyn Evaluator>, FindItError> {
    if args.len() > 1 {
        return Err(FindItError::BadExpression(
            "Length must have only one argument".into(),
        ));
    }
    let Some(str) = args.pop_front() else {
        return Err(FindItError::BadExpression(
            "Length must have one argument".into(),
        ));
    };
    if str.expected_type() != ValueType::String {
        return Err(FindItError::BadExpression(
            "Length can only work with strings".into(),
        ));
    }
    Ok(Box::new(Length { str }))
}

struct Length {
    str: Box<dyn Evaluator>,
}
impl Evaluator for Length {
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::String(str) = self.str.eval(file) else {
            return Value::Empty;
        };
        str.len().into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Number
    }
}

pub(crate) fn new_lower(
    mut args: VecDeque<Box<dyn Evaluator>>,
) -> Result<Box<dyn Evaluator>, FindItError> {
    if args.len() > 1 {
        return Err(FindItError::BadExpression(
            "Lower must have only one argument".into(),
        ));
    }
    let Some(str) = args.pop_front() else {
        return Err(FindItError::BadExpression(
            "Length must have one argument".into(),
        ));
    };
    if str.expected_type() != ValueType::String {
        return Err(FindItError::BadExpression(
            "Length can only work with strings".into(),
        ));
    }
    Ok(Box::new(Lower { str }))
}

struct Lower {
    str: Box<dyn Evaluator>,
}
impl Evaluator for Lower {
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::String(str) = self.str.eval(file) else {
            return Value::Empty;
        };
        str.to_lowercase().into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::String
    }
}

pub(crate) fn new_upper(
    mut args: VecDeque<Box<dyn Evaluator>>,
) -> Result<Box<dyn Evaluator>, FindItError> {
    if args.len() > 1 {
        return Err(FindItError::BadExpression(
            "Upper must have only one argument".into(),
        ));
    }
    let Some(str) = args.pop_front() else {
        return Err(FindItError::BadExpression(
            "Upper must have one argument".into(),
        ));
    };
    if str.expected_type() != ValueType::String {
        return Err(FindItError::BadExpression(
            "Upper can only work with strings".into(),
        ));
    }
    Ok(Box::new(Upper { str }))
}

struct Upper {
    str: Box<dyn Evaluator>,
}
impl Evaluator for Upper {
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::String(str) = self.str.eval(file) else {
            return Value::Empty;
        };
        str.to_uppercase().into()
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

    #[test]
    fn length_no_string_expr() {
        let err = read_expr("LEN(12)").err();
        assert!(err.is_some())
    }

    #[test]
    fn length_no_args() {
        let err = read_expr("LEN()").err();
        assert!(err.is_some())
    }

    #[test]
    fn length_too_many_args() {
        let err = read_expr("len(\"abc\", \"def\")").err();
        assert!(err.is_some())
    }

    #[test]
    fn length_null_str_return_empty() {
        let eval = read_expr("len(content)").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn length_expect_number() {
        let expr = read_expr("len(\"test\")").unwrap();
        assert_eq!(expr.expected_type(), ValueType::Number);
    }

    #[test]
    fn length_return_the_correct_value() {
        let eval = read_expr("len(\"123\")").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Number(3))
    }

    #[test]
    fn lower_no_string_expr() {
        let err = read_expr("Lower(12)").err();
        assert!(err.is_some())
    }

    #[test]
    fn lower_no_args() {
        let err = read_expr("lower()").err();
        assert!(err.is_some())
    }

    #[test]
    fn lower_too_many_args() {
        let err = read_expr("lower(\"abc\", \"def\")").err();
        assert!(err.is_some())
    }

    #[test]
    fn lower_null_str_return_empty() {
        let eval = read_expr("lower(content)").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn lower_expect_string() {
        let expr = read_expr("lower(\"test\")").unwrap();
        assert_eq!(expr.expected_type(), ValueType::String);
    }

    #[test]
    fn lower_return_the_correct_value() {
        let eval = read_expr("lower(\"abcDEF\")").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::String("abcdef".into()))
    }

    #[test]
    fn upper_no_string_expr() {
        let err = read_expr("Upper(12)").err();
        assert!(err.is_some())
    }

    #[test]
    fn upper_no_args() {
        let err = read_expr("upper()").err();
        assert!(err.is_some())
    }

    #[test]
    fn upper_too_many_args() {
        let err = read_expr("upper(\"abc\", \"def\")").err();
        assert!(err.is_some())
    }

    #[test]
    fn upper_null_str_return_empty() {
        let eval = read_expr("upper(content)").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn upper_expect_string() {
        let expr = read_expr("upper(\"test\")").unwrap();
        assert_eq!(expr.expected_type(), ValueType::String);
    }

    #[test]
    fn upper_return_the_correct_value() {
        let eval = read_expr("upper(\"abcDEF\")").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::String("ABCDEF".into()))
    }

    #[test]
    fn replace_return_the_correct_value() {
        let eval = read_expr("replace(\"abc123def123\" from \"12\" to \" 12 \")").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::String("abc 12 3def 12 3".into()))
    }

    #[test]
    fn replace_return_the_correct_value_when_replacement_is_invalid_regex() {
        let eval = read_expr("replace(\"[[--]]\" from \"[\" to \".\")").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::String("..--]]".into()))
    }

    #[test]
    fn replace_return_the_correct_value_with_pattern() {
        let eval =
            read_expr("replace(\"abc 123 def 2345 gsr 23\" pattern \"[0-9]+\" to \"<numbers>\")")
                .unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);

        assert_eq!(
            value,
            Value::String("abc <numbers> def <numbers> gsr <numbers>".into())
        )
    }

    #[test]
    fn replace_return_empty_if_source_is_not_a_string() {
        let eval = read_expr("replace(content from \"12\" to \" 12 \")").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn replace_return_empty_if_substr_is_not_a_string() {
        let eval = read_expr("replace(\"abc123def123\" from content to \" 12 \")").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn replace_return_empty_if_the_replacement_is_not_a_string() {
        let eval = read_expr("replace(\"abc123def123\" from \"12\" to content)").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn replace_with_pattern_return_empty_if_source_is_not_a_string() {
        let eval = read_expr("replace(content pattern \"12\" to \" 12 \")").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn replace_with_pattern_return_empty_if_substr_is_not_a_string() {
        let eval = read_expr("replace(\"abc123def123\" pattern content to \" 12 \")").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn replace_with_pattern_return_empty_if_the_replacement_is_not_a_string() {
        let eval = read_expr("replace(\"abc123def123\" pattern \"12\" to content)").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn replace_with_pattern_return_empty_if_the_pattern_is_invalid() {
        let eval = read_expr("replace(\"abc123def123\" pattern \"[\" to \"-\")").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn replace_return_the_correct_expected_value() {
        let eval = read_expr("replace(\"abc123def123\" from \"12\" to \" 12 \")").unwrap();

        assert_eq!(eval.expected_type(), ValueType::String)
    }

    #[test]
    fn replace_with_pattern_return_the_correct_expected_value() {
        let eval = read_expr("replace(\"abc123def123\" pattern \"12\" to \" 12 \")").unwrap();

        assert_eq!(eval.expected_type(), ValueType::String)
    }

    #[test]
    fn replace_fails_when_source_is_not_a_string() {
        let err = read_expr("replace(1 from \"12\" to \" 12 \")").err();

        assert!(err.is_some())
    }

    #[test]
    fn replace_fails_when_from_is_not_a_string() {
        let err = read_expr("replace(\"1\" from 12 to \" 12 \")").err();

        assert!(err.is_some())
    }

    #[test]
    fn replace_fails_when_pattern_is_not_a_string() {
        let err = read_expr("replace(\"1\" pattern 12 to \" 12 \")").err();

        assert!(err.is_some())
    }

    #[test]
    fn replace_fails_when_to_is_not_a_string() {
        let err = read_expr("replace(\"1\" pattern \"12\" to  12 )").err();

        assert!(err.is_some())
    }
}
