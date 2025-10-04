use crate::errors::FindItError;
use crate::evaluators::expr::{Evaluator, get_eval};
use crate::file_wrapper::FileWrapper;
use crate::parser::ast::format::Format as FormatExpression;
use crate::value::{Value, ValueType};

impl TryFrom<&FormatExpression> for Box<dyn Evaluator> {
    type Error = FindItError;
    fn try_from(format: &FormatExpression) -> Result<Self, Self::Error> {
        let timestamp = get_eval(&format.timestamp)?;
        if timestamp.expected_type() != ValueType::Date {
            return Err(FindItError::BadExpression("Can only format dates".into()));
        }
        let format = get_eval(&format.format)?;
        if format.expected_type() != ValueType::String {
            return Err(FindItError::BadExpression(
                "Format must be a string value".into(),
            ));
        }

        Ok(Box::new(Format { timestamp, format }))
    }
}

struct Format {
    timestamp: Box<dyn Evaluator>,
    format: Box<dyn Evaluator>,
}
impl Evaluator for Format {
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::Date(timestamp) = self.timestamp.eval(file) else {
            return Value::Empty;
        };
        let Value::String(format) = self.format.eval(file) else {
            return Value::Empty;
        };
        let mut str = String::new();
        if timestamp
            .format(format.as_str())
            .write_to(&mut str)
            .is_err()
        {
            Value::Empty
        } else {
            str.into()
        }
    }
    fn expected_type(&self) -> ValueType {
        ValueType::String
    }
}

#[cfg(test)]
mod tests {

    use std::path::PathBuf;

    use crate::{
        evaluators::expr::read_expr,
        file_wrapper::FileWrapper,
        value::{Value, ValueType},
    };

    #[test]
    fn format_no_date() {
        let err = read_expr("format(10 as \"%Y\")").err();
        assert!(err.is_some())
    }

    #[test]
    fn format_no_format() {
        let err = read_expr("format([2021-12-21] as false)").err();
        assert!(err.is_some())
    }

    #[test]
    fn format_expected_type() {
        let expr = read_expr("format([2021-12-21] as \"%Y\")").unwrap();

        assert_eq!(expr.expected_type(), ValueType::String);
    }

    #[test]
    fn format_return_value() {
        let expr = read_expr("format([2021-12-21] as \"%Y\")").unwrap();
        let wrapper = FileWrapper::new(PathBuf::new(), 1);

        assert_eq!(expr.eval(&wrapper), Value::String("2021".into()));
    }

    #[test]
    fn format_date_return_value() {
        let expr = read_expr("formatDate([2021-12-21] as \"%d\")").unwrap();
        let wrapper = FileWrapper::new(PathBuf::new(), 1);

        assert_eq!(expr.eval(&wrapper), Value::String("21".into()));
    }

    #[test]
    fn format_return_nothing_for_invalid_format() {
        let expr = read_expr("format([2021-12-21] as \"%\")").unwrap();
        let wrapper = FileWrapper::new(PathBuf::new(), 1);

        assert_eq!(expr.eval(&wrapper), Value::Empty);
    }

    #[test]
    fn format_return_nothing_for_no_date() {
        let expr = read_expr("format(modified as \"%Y\")").unwrap();
        let wrapper = FileWrapper::new(PathBuf::new(), 1);

        assert_eq!(expr.eval(&wrapper), Value::Empty);
    }

    #[test]
    fn format_return_nothing_for_no_format() {
        let expr = read_expr("format([2021-12-21] as content)").unwrap();
        let wrapper = FileWrapper::new(PathBuf::new(), 1);

        assert_eq!(expr.eval(&wrapper), Value::Empty);
    }
}
