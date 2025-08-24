use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, offset::LocalResult};
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
    val: DateTime<Local>,
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

    let naive_date_formats = ["%d/%b/%Y", "%Y-%m-%d"];
    for format in naive_date_formats {
        if let Ok(date) = NaiveDate::parse_from_str(val, format)
            && let LocalResult::Single(date) =
                date.and_hms_opt(0, 0, 0).unwrap().and_local_timezone(Local)
        {
            return Box::new(DateLiteral { val: date });
        }
    }

    let naive_date_formats = [
        "%d/%b/%Y %H:%M",
        "%d/%b/%Y %H:%M:%S",
        "%d/%b/%Y %H:%M:%S%.f",
        "%Y-%m-%d %H:%M",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M:%S%.f",
    ];

    for format in naive_date_formats {
        if let Ok(date) = NaiveDateTime::parse_from_str(val, format)
            && let LocalResult::Single(date) = date.and_local_timezone(Local)
        {
            return Box::new(DateLiteral { val: date });
        }
    }

    let naive_date_formats_with_tz = [
        "%d/%b/%Y %H:%M %z",
        "%d/%b/%Y %H:%M:%S %z",
        "%d/%b/%Y %H:%M:%S%.f %z",
        "%Y-%m-%d %H:%M %z",
        "%Y-%m-%d %H:%M:%S %z",
        "%Y-%m-%d %H:%M:%S%.f %z",
    ];

    for format in naive_date_formats_with_tz {
        if let Ok(date) = DateTime::parse_from_str(val, format) {
            return Box::new(DateLiteral { val: date.into() });
        }
    }

    Box::new(StringLiteral {
        val: val.to_string(),
    })
}

#[cfg(test)]
mod tests {

    use std::path::Path;

    use chrono::{FixedOffset, Local, MappedLocalTime, NaiveDate, NaiveTime, TimeZone, Utc};

    use crate::{
        expr::read_expr,
        file_wrapper::FileWrapper,
        value::{Value, ValueType},
    };

    #[test]
    fn unsupported_literal() {
        let err = read_expr("0x432").err();
        assert!(err.is_some())
    }

    #[test]
    fn numeric_literal() {
        let eval = read_expr("432").unwrap();
        let path = Path::new(".");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Number(432))
    }

    #[test]
    fn unsupported_numeric_literal() {
        let err = read_expr("432.443").err();
        assert!(err.is_some())
    }

    #[test]
    fn boolean_literal() {
        let eval = read_expr("TRUE").unwrap();
        let path = Path::new(".");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Bool(true))
    }

    #[test]
    fn null_literal() {
        let eval = read_expr("null").unwrap();
        let path = Path::new(".");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }
    #[test]
    fn null_literal_type() {
        let eval = read_expr("null").unwrap();
        assert_eq!(eval.expected_type(), ValueType::Empty);
    }

    #[test]
    fn string_literal() {
        let eval = read_expr("'hello'").unwrap();
        let path = Path::new(".");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::String("hello".to_string()))
    }

    fn date_literal_tz(
        date_as_text: &str,
        expected_date: NaiveDate,
        expected_time: NaiveTime,
        time_zone: impl TimeZone,
    ) {
        let MappedLocalTime::Single(expected_date) = expected_date
            .and_time(expected_time)
            .and_local_timezone(time_zone)
        else {
            panic!("Invalid date");
        };
        let expected_date = expected_date.with_timezone(&Local);
        let eval = read_expr(&format!("'{date_as_text}'")).unwrap();
        let path = Path::new(".");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Date(expected_date))
    }

    fn date_literal(date_as_text: &str, expected_date: NaiveDate, expected_time: NaiveTime) {
        date_literal_tz(date_as_text, expected_date, expected_time, Local)
    }

    #[test]
    fn date_literal_with_slash() {
        date_literal(
            "20/Jan/2025",
            NaiveDate::from_ymd_opt(2025, 1, 20).unwrap(),
            NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
        );
    }

    #[test]
    fn date_literal_with_dash() {
        date_literal(
            "2025-03-17",
            NaiveDate::from_ymd_opt(2025, 3, 17).unwrap(),
            NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
        );
    }

    #[test]
    fn date_literal_with_slash_hm() {
        date_literal(
            "20/Jan/2025 11:43",
            NaiveDate::from_ymd_opt(2025, 1, 20).unwrap(),
            NaiveTime::from_hms_opt(11, 43, 0).unwrap(),
        );
    }

    #[test]
    fn date_literal_with_dash_hm() {
        date_literal(
            "2025-03-17 15:21",
            NaiveDate::from_ymd_opt(2025, 3, 17).unwrap(),
            NaiveTime::from_hms_opt(15, 21, 0).unwrap(),
        );
    }

    #[test]
    fn date_literal_with_slash_hms() {
        date_literal(
            "20/Jan/2025 11:43:14",
            NaiveDate::from_ymd_opt(2025, 1, 20).unwrap(),
            NaiveTime::from_hms_opt(11, 43, 14).unwrap(),
        );
    }

    #[test]
    fn date_literal_with_dash_hms() {
        date_literal(
            "2025-03-17 15:21:54",
            NaiveDate::from_ymd_opt(2025, 3, 17).unwrap(),
            NaiveTime::from_hms_opt(15, 21, 54).unwrap(),
        );
    }

    #[test]
    fn date_literal_with_dash_hmsms() {
        date_literal(
            "2025-03-17 15:21:54.3",
            NaiveDate::from_ymd_opt(2025, 3, 17).unwrap(),
            NaiveTime::from_hms_milli_opt(15, 21, 54, 300).unwrap(),
        );
    }

    #[test]
    fn date_literal_with_slash_hmsms() {
        date_literal(
            "11/Aug/1976 17:45:00.421",
            NaiveDate::from_ymd_opt(1976, 8, 11).unwrap(),
            NaiveTime::from_hms_milli_opt(17, 45, 0, 421).unwrap(),
        );
    }

    #[test]
    fn date_literal_with_slash_hmtz() {
        let offset = FixedOffset::east_opt(4 * 3600).unwrap();

        date_literal_tz(
            "21/Nov/2031 12:21 +0400",
            NaiveDate::from_ymd_opt(2031, 11, 21).unwrap(),
            NaiveTime::from_hms_opt(12, 21, 0).unwrap(),
            offset,
        );
    }

    #[test]
    fn date_literal_with_slash_hmstz() {
        let offset = FixedOffset::west_opt(5 * 3600).unwrap();

        date_literal_tz(
            "12/May/1986 14:31:12 -0500",
            NaiveDate::from_ymd_opt(1986, 5, 12).unwrap(),
            NaiveTime::from_hms_opt(14, 31, 12).unwrap(),
            offset,
        );
    }

    #[test]
    fn date_literal_with_slash_hmsmtz() {
        let offset = FixedOffset::west_opt(3600).unwrap();

        date_literal_tz(
            "12/Feb/2025 14:31:12.40 -0100",
            NaiveDate::from_ymd_opt(2025, 2, 12).unwrap(),
            NaiveTime::from_hms_milli_opt(14, 31, 12, 400).unwrap(),
            offset,
        );
    }

    #[test]
    fn date_literal_with_dash_hmtz() {
        let offset = FixedOffset::west_opt(3 * 3600).unwrap();

        date_literal_tz(
            "2024-10-09 16:12 -0300",
            NaiveDate::from_ymd_opt(2024, 10, 9).unwrap(),
            NaiveTime::from_hms_opt(16, 12, 0).unwrap(),
            offset,
        );
    }

    #[test]
    fn date_literal_with_dash_hmstz() {
        let offset = FixedOffset::east_opt(5 * 3600).unwrap();

        date_literal_tz(
            "1986-5-12 14:31:12 +0500",
            NaiveDate::from_ymd_opt(1986, 5, 12).unwrap(),
            NaiveTime::from_hms_opt(14, 31, 12).unwrap(),
            offset,
        );
    }

    #[test]
    fn date_literal_with_dash_hmsmtz() {
        let offset = FixedOffset::east_opt(3600).unwrap();

        date_literal_tz(
            "2025-02-12 14:31:12.40 +0100",
            NaiveDate::from_ymd_opt(2025, 2, 12).unwrap(),
            NaiveTime::from_hms_milli_opt(14, 31, 12, 400).unwrap(),
            offset,
        );
    }

    #[test]
    fn date_literal_with_rfc3339() {
        date_literal_tz(
            "1985-04-12T23:20:50.52Z",
            NaiveDate::from_ymd_opt(1985, 4, 12).unwrap(),
            NaiveTime::from_hms_milli_opt(23, 20, 50, 520).unwrap(),
            Utc,
        );
    }
}
