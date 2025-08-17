use chrono::{DateTime, Utc};
use sqlparser::ast::Value as SqlValue;

use crate::{
    errors::FindItError,
    expr::Evaluator,
    file_wrapper::FileWrapper,
    value::{Value, ValueType},
};

pub(crate) fn new_literal_value(value: &SqlValue) -> Result<Box<dyn Evaluator>, FindItError> {
    match value {
        SqlValue::Number(num, _) => Ok(Box::new(NumberLiteral::new(num)?)),
        SqlValue::Boolean(b) => Ok(Box::new(BooleanLiteral { val: *b })),
        SqlValue::Null => Ok(Box::new(EmptyLiteral {})),
        SqlValue::SingleQuotedString(str) => Ok(parse_date_or_string(str)),
        _ => Err(FindItError::BadExpression(format!(
            "Unsupported literal value: {value}"
        ))),
    }
}

struct NumberLiteral {
    num: u64,
}
impl Evaluator for NumberLiteral {
    fn eval(&self, _: &FileWrapper) -> Value {
        Value::Number(self.num)
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Number
    }
}
impl NumberLiteral {
    fn new(num: &str) -> Result<Self, FindItError> {
        let num = num.parse::<u64>()?;
        Ok(Self { num })
    }
}
struct BooleanLiteral {
    val: bool,
}
impl Evaluator for BooleanLiteral {
    fn eval(&self, _: &FileWrapper) -> Value {
        Value::Bool(self.val)
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Bool
    }
}
struct EmptyLiteral {}
impl Evaluator for EmptyLiteral {
    fn eval(&self, _: &FileWrapper) -> Value {
        Value::Empty
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Empty
    }
}
struct StringLiteral {
    val: String,
}
impl Evaluator for StringLiteral {
    fn eval(&self, _: &FileWrapper) -> Value {
        Value::String(self.val.clone())
    }
    fn expected_type(&self) -> ValueType {
        ValueType::String
    }
}
struct DateLiteral {
    val: DateTime<Utc>,
}
impl Evaluator for DateLiteral {
    fn eval(&self, _: &FileWrapper) -> Value {
        Value::Date(self.val)
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Date
    }
}

fn parse_date_or_string(val: &str) -> Box<dyn Evaluator> {
    if let Ok(date) = DateTime::parse_from_rfc3339(val) {
        return Box::new(DateLiteral { val: date.into() });
    }

    let date_formats = [
        "%d/%b/%Y %H:%M:%S",
        "%d/%b/%Y %H:%M:%S %Z",
        "%d/%b/%Y",
        "%Y-%m-%d %H:%M:%S%.f",
        "%Y-%m-%d %H:%M:%S%.f %Z",
        "%Y-%m-%d",
    ];
    for format in date_formats {
        if let Ok(date) = DateTime::parse_from_str(val, format) {
            return Box::new(DateLiteral { val: date.into() });
        }
    }

    Box::new(StringLiteral {
        val: val.to_string(),
    })
}
