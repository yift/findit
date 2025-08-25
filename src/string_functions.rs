use std::collections::HashSet;

use regex::Regex;
use sqlparser::ast::{Expr, TrimWhereField};

use crate::{
    errors::FindItError,
    expr::{Evaluator, get_eval},
    file_wrapper::FileWrapper,
    value::{Value, ValueType},
};

pub(crate) fn new_regex(
    expr: &Expr,
    pattern: &Expr,
    negate: bool,
) -> Result<Box<dyn Evaluator>, FindItError> {
    let expr = get_eval(expr)?;
    if expr.expected_type() != ValueType::String {
        return Err(FindItError::BadExpression(
            "REGULAR expressions can only work with strings".into(),
        ));
    }
    let pattern = get_eval(pattern)?;
    if pattern.expected_type() != ValueType::String {
        return Err(FindItError::BadExpression(
            "REGULAR expressions pattern can only be strings".into(),
        ));
    }
    Ok(Box::new(Regexp {
        expr,
        pattern,
        negate,
    }))
}

struct Regexp {
    expr: Box<dyn Evaluator>,
    pattern: Box<dyn Evaluator>,
    negate: bool,
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
        (self.negate ^ regexp.is_match(&expr)).into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Bool
    }
}

pub(crate) fn new_position(str: &Expr, sub_str: &Expr) -> Result<Box<dyn Evaluator>, FindItError> {
    let str = get_eval(str)?;
    let sub_str = get_eval(sub_str)?;

    if (str.expected_type(), sub_str.expected_type()) != (ValueType::String, ValueType::String) {
        return Err(FindItError::BadExpression(
            "POSITION can only work with strings".into(),
        ));
    }
    Ok(Box::new(Position { str, sub_str }))
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

pub(crate) fn new_substring(
    str: &Expr,
    from: &Option<Box<Expr>>,
    length: &Option<Box<Expr>>,
) -> Result<Box<dyn Evaluator>, FindItError> {
    let str = get_eval(str)?;
    if str.expected_type() != ValueType::String {
        return Err(FindItError::BadExpression(
            "SUBSTRING can only work with strings".into(),
        ));
    }
    let from = if let Some(from) = from {
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
    let length = if let Some(length) = length {
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
    if length.is_none() && from.is_none() {
        return Err(FindItError::BadExpression(
            "SUBSTRING must have at least FROM or FOR".into(),
        ));
    }
    Ok(Box::new(SubString { str, from, length }))
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
            let Ok(from) = from.try_into() else {
                return Value::Empty;
            };
            if from > str.len() {
                return "".into();
            }
            let from = if from == 0 { 1 } else { from };
            str = &str[from - 1..];
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

pub(crate) fn new_trim(
    str: &Expr,
    trim_where: &Option<TrimWhereField>,
    what: &Option<Box<Expr>>,
) -> Result<Box<dyn Evaluator>, FindItError> {
    let str = get_eval(str)?;
    if str.expected_type() != ValueType::String {
        return Err(FindItError::BadExpression(
            "TRIM can only work with strings".into(),
        ));
    }
    let trim_where = trim_where.unwrap_or(TrimWhereField::Both);
    let what = if let Some(what) = what {
        let what = get_eval(what)?;
        if what.expected_type() != ValueType::String {
            return Err(FindItError::BadExpression(
                "TRIM can only trim strings".into(),
            ));
        }
        Some(what)
    } else {
        None
    };
    Ok(Box::new(Trim {
        str,
        trim_where,
        what,
    }))
}

struct Trim {
    str: Box<dyn Evaluator>,
    trim_where: TrimWhereField,
    what: Option<Box<dyn Evaluator>>,
}
impl Evaluator for Trim {
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::String(str) = self.str.eval(file) else {
            return Value::Empty;
        };
        if let Some(what) = &self.what {
            let Value::String(what) = what.eval(file) else {
                return Value::Empty;
            };
            if what.is_empty() {
                return str.into();
            }
            let mut chars = HashSet::new();
            for c in what.chars() {
                chars.insert(c);
            }
            match self.trim_where {
                TrimWhereField::Leading => str.trim_start_matches(|c| chars.contains(&c)),
                TrimWhereField::Trailing => str.trim_end_matches(|c| chars.contains(&c)),
                TrimWhereField::Both => str.trim_matches(|c| chars.contains(&c)),
            }
        } else {
            match self.trim_where {
                TrimWhereField::Leading => str.trim_start(),
                TrimWhereField::Trailing => str.trim_end(),
                TrimWhereField::Both => str.trim(),
            }
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
        expr::read_expr,
        file_wrapper::FileWrapper,
        value::{Value, ValueType},
    };

    #[test]
    fn regex_no_string_expr() {
        let err = read_expr("1 RLIKE 'a'").err();
        assert!(err.is_some())
    }

    #[test]
    fn regex_no_string_pattern() {
        let err = read_expr("'a' RLIKE 1").err();
        assert!(err.is_some())
    }

    #[test]
    fn regex_null_expr_return_empty() {
        let eval = read_expr("content RLIKE 'abc'").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn regex_null_pattern_return_empty() {
        let eval = read_expr("'abc' RLIKE content").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn regex_bad_pattern_return_empty() {
        let eval = read_expr("'abc' RLIKE '['").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn position_no_string_expr() {
        let err = read_expr("POSITION('txt' IN 12)").err();
        assert!(err.is_some())
    }

    #[test]
    fn position_no_string_str() {
        let err = read_expr("POSITION(12 IN path)").err();
        assert!(err.is_some())
    }

    #[test]
    fn position_expect_number() {
        let expr = read_expr("POSITION('a' IN path)").unwrap();
        assert_eq!(expr.expected_type(), ValueType::Number);
    }

    #[test]
    fn position_null_expr_return_empty() {
        let eval = read_expr("POSITION(content IN 'abc')").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn position_null_str_return_empty() {
        let eval = read_expr("POSITION('abc' IN content)").unwrap();
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
        let eval = read_expr("SUBSTRING('abc' FOR length)").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn substring_null_from_return_empty() {
        let eval = read_expr("SUBSTRING('abc' FROM length)").unwrap();
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
    fn trim_no_string_chars() {
        let err = read_expr("TRIM(12 FROM 'text')").err();
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
    fn trim_null_chars_return_empty() {
        let eval = read_expr("TRIM(content FROM 'abc')").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn trim_empty_chars_return_text() {
        let eval = read_expr("TRIM('' FROM 'abc')").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, "abc".into())
    }

    #[test]
    fn trim_expect_string() {
        let expr = read_expr("TRIM('')").unwrap();
        assert_eq!(expr.expected_type(), ValueType::String);
    }
}
