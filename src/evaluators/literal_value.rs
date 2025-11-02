use crate::{
    evaluators::expr::Evaluator,
    file_wrapper::FileWrapper,
    value::{Value, ValueType},
};

impl From<&Value> for Box<dyn Evaluator> {
    fn from(value: &Value) -> Self {
        Box::new(value.clone())
    }
}

impl Evaluator for Value {
    fn eval(&self, _: &FileWrapper) -> Value {
        self.clone()
    }
    fn expected_type(&self) -> ValueType {
        match self {
            Value::Bool(_) => ValueType::Bool,
            Value::Date(_) => ValueType::Date,
            Value::Number(_) => ValueType::Number,
            Value::String(_) => ValueType::String,
            Value::Path(_) => ValueType::Path,
            _ => ValueType::Empty,
        }
    }
}

#[cfg(test)]
mod tests {

    use std::path::Path;

    use chrono::{FixedOffset, Local, MappedLocalTime, NaiveDate, NaiveTime, TimeZone, Utc};

    use crate::{evaluators::expr::read_expr, file_wrapper::FileWrapper, value::Value};

    #[test]
    fn numeric_literal() {
        let eval = read_expr("432").unwrap();
        let path = Path::new(".");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Number(432))
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
    fn string_literal() {
        let eval = read_expr("\"hello\"").unwrap();
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
        let eval = read_expr(&format!("[{date_as_text}]")).unwrap();
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
