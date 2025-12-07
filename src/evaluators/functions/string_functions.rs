use regex::Regex;

use crate::{
    errors::FindItError,
    evaluators::expr::{BindingsTypes, Evaluator, EvaluatorFactory},
    file_wrapper::FileWrapper,
    parser::ast::replace::{Replace, ReplaceWhat},
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
