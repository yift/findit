use regex::Regex;

use crate::{
    errors::FindItError,
    evaluators::expr::{BindingsTypes, Evaluator, EvaluatorFactory},
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

impl EvaluatorFactory for PositionExpression {
    fn build(&self, bindings: &BindingsTypes) -> Result<Box<dyn Evaluator>, FindItError> {
        let str = self.super_string.build(bindings)?;
        let sub_str = self.sub_string.build(bindings)?;

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

impl EvaluatorFactory for Replace {
    fn build(&self, bindings: &BindingsTypes) -> Result<Box<dyn Evaluator>, FindItError> {
        let source = self.source.build(bindings)?;
        let (what, regex) = match &self.what {
            ReplaceWhat::Pattern(p) => (p.build(bindings)?, true),
            ReplaceWhat::String(p) => (p.build(bindings)?, false),
        };
        let to = self.to.build(bindings)?;

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

impl EvaluatorFactory for Substring {
    fn build(&self, bindings: &BindingsTypes) -> Result<Box<dyn Evaluator>, FindItError> {
        let str = self.super_string.build(bindings)?;
        if str.expected_type() != ValueType::String {
            return Err(FindItError::BadExpression(
                "SUBSTRING can only work with strings".into(),
            ));
        }
        let from = if let Some(from) = &self.substring_from {
            let from = from.build(bindings)?;
            if from.expected_type() != ValueType::Number {
                return Err(FindItError::BadExpression(
                    "SUBSTRING can only start from a number".into(),
                ));
            }
            Some(from)
        } else {
            None
        };
        let length = if let Some(length) = &self.substring_for {
            let length = length.build(bindings)?;
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
        let eval = read_expr("SUBSTRING(\"abc\" FOR length())").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn substring_null_from_return_empty() {
        let eval = read_expr("SUBSTRING(\"abc\" FROM length())").unwrap();
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
