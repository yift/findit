use chrono::offset::LocalResult;
use chrono::{DateTime, Local, NaiveDate, NaiveDateTime};

use crate::errors::FindItError;
use crate::evaluators::expr::{BindingsTypes, Evaluator, EvaluatorFactory};
use crate::file_wrapper::FileWrapper;
use crate::parser::ast::parse::Parse as ParseExpression;
use crate::value::{Value, ValueType};

impl EvaluatorFactory for ParseExpression {
    fn build(&self, bindings: &BindingsTypes) -> Result<Box<dyn Evaluator>, FindItError> {
        let str = self.str.build(bindings)?;
        if str.expected_type() != ValueType::String {
            return Err(FindItError::BadExpression("Can only parse strings".into()));
        }
        let format = self.format.build(bindings)?;
        if format.expected_type() != ValueType::String {
            return Err(FindItError::BadExpression(
                "Parse format must be a string value".into(),
            ));
        }

        Ok(Box::new(Parse { str, format }))
    }
}

struct Parse {
    str: Box<dyn Evaluator>,
    format: Box<dyn Evaluator>,
}
impl Evaluator for Parse {
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::String(str) = self.str.eval(file) else {
            return Value::Empty;
        };
        let Value::String(format) = self.format.eval(file) else {
            return Value::Empty;
        };

        if let Ok(timestamp) = DateTime::parse_from_str(str.as_str(), format.as_str()) {
            Value::Date(timestamp.into())
        } else if let Ok(date) = NaiveDateTime::parse_from_str(str.as_str(), format.as_str())
            && let LocalResult::Single(date) = date.and_local_timezone(Local)
        {
            Value::Date(date)
        } else if let Ok(date) = NaiveDate::parse_from_str(str.as_str(), format.as_str())
            && let LocalResult::Single(date) =
                date.and_hms_opt(0, 0, 0).unwrap().and_local_timezone(Local)
        {
            Value::Date(date)
        } else {
            Value::Empty
        }
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Date
    }
}

#[cfg(test)]
mod tests {

    use std::path::PathBuf;

    use chrono::{FixedOffset, NaiveDate, NaiveTime};

    use crate::{
        evaluators::expr::read_expr,
        file_wrapper::FileWrapper,
        value::{Value, ValueType},
    };

    #[test]
    fn parse_no_date() {
        let err = read_expr("parse(10 from \"%Y\")").err();
        assert!(err.is_some())
    }

    #[test]
    fn parse_no_format() {
        let err = read_expr("parse(\"2021-12-21\" from false)").err();
        assert!(err.is_some())
    }

    #[test]
    fn parse_expected_type() {
        let expr = read_expr("parse(\"2021-12-21\" from \"%Y\")").unwrap();

        assert_eq!(expr.expected_type(), ValueType::Date);
    }

    #[test]
    fn parse_return_value() {
        let expr = read_expr("parse(\"2021-12-21\" from \"%Y-%m-%d\") as string").unwrap();
        let wrapper = FileWrapper::new(PathBuf::new(), 1);

        assert_eq!(
            expr.eval(&wrapper),
            Value::String("21/Dec/2021 00:00:00".into())
        );
    }

    #[test]
    fn parse_return_value_with_time() {
        let expr =
            read_expr("parseDate(\"21:11:54=>2024-10-21\" from \"%H:%M:%S=>%Y-%m-%d\") as string")
                .unwrap();
        let wrapper = FileWrapper::new(PathBuf::new(), 1);

        assert_eq!(
            expr.eval(&wrapper),
            Value::String("21/Oct/2024 21:11:54".into())
        );
    }

    #[test]
    fn parse_return_value_with_time_and_timezone() {
        let expected = NaiveDate::from_ymd_opt(2025, 8, 21)
            .unwrap()
            .and_time(NaiveTime::from_hms_opt(4, 21, 10).unwrap())
            .and_local_timezone(FixedOffset::west_opt(3 * 3600).unwrap())
            .unwrap();
        let expr =
            read_expr("parseDate(\"4:21:10 -0300; 21/8/2025\" from \"%H:%M:%S %z; %d/%m/%Y\")")
                .unwrap();
        let wrapper = FileWrapper::new(PathBuf::new(), 1);

        assert_eq!(expr.eval(&wrapper), Value::Date(expected.into()));
    }

    #[test]
    fn parse_return_value_with_bad_format() {
        let expr = read_expr("parseDate(\"21:11:54=>2024-10-21\" from \"nop\")").unwrap();
        let wrapper = FileWrapper::new(PathBuf::new(), 1);

        assert_eq!(expr.eval(&wrapper), Value::Empty);
    }

    #[test]
    fn parse_return_value_with_bad_time() {
        let expr = read_expr("parseDate(\"nop\" from \"%H:%M:%S=>%Y-%m-%d\")").unwrap();
        let wrapper = FileWrapper::new(PathBuf::new(), 1);

        assert_eq!(expr.eval(&wrapper), Value::Empty,);
    }

    #[test]
    fn parse_return_nothing_with_empty_string() {
        let expr = read_expr("parse(name from \"%Y-%m-%d\")").unwrap();
        let wrapper = FileWrapper::new(PathBuf::new(), 1);

        assert_eq!(expr.eval(&wrapper), Value::Empty);
    }

    #[test]
    fn parse_return_nothing_with_empty_format() {
        let expr = read_expr("parse(\"2021-12-21\" from content)").unwrap();
        let wrapper = FileWrapper::new(PathBuf::new(), 1);

        assert_eq!(expr.eval(&wrapper), Value::Empty);
    }
}
