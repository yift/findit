use crate::{
    errors::FindItError,
    expr::{Evaluator, get_eval},
    file_wrapper::FileWrapper,
    parser::{negate::Negate as NegateExpression, self_divide::SelfDivide},
    value::{Value, ValueType},
};

impl TryFrom<&NegateExpression> for Box<dyn Evaluator> {
    type Error = FindItError;
    fn try_from(value: &NegateExpression) -> Result<Self, Self::Error> {
        let expr = get_eval(&value.expression)?;
        if expr.expected_type() != ValueType::Bool {
            Err(FindItError::BadExpression(
                "NOT can only applied to boolean expressions".into(),
            ))
        } else {
            Ok(Box::new(Negate { expr }))
        }
    }
}
pub(crate) fn make_negate(expr: Box<dyn Evaluator>) -> Box<dyn Evaluator> {
    Box::new(Negate { expr })
}

struct Negate {
    expr: Box<dyn Evaluator>,
}
impl Evaluator for Negate {
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::Bool(val) = self.expr.eval(file) else {
            return Value::Empty;
        };
        (!val).into()
    }
    fn expected_type(&self) -> crate::value::ValueType {
        ValueType::Bool
    }
}

impl TryFrom<&SelfDivide> for Box<dyn Evaluator> {
    type Error = FindItError;
    fn try_from(value: &SelfDivide) -> Result<Self, Self::Error> {
        let expr = get_eval(&value.expression)?;
        match expr.expected_type() {
            ValueType::String | ValueType::Path => Ok(Box::new(AccessFile { expr })),
            _ => Err(FindItError::BadExpression(
                "/ can only be applied to string or path".into(),
            )),
        }
    }
}
struct AccessFile {
    expr: Box<dyn Evaluator>,
}

impl Evaluator for AccessFile {
    fn expected_type(&self) -> ValueType {
        ValueType::Path
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let file = match self.expr.eval(file) {
            Value::String(str) => file.path().join(str),
            Value::Path(path) => file.path().join(path),
            _ => {
                return Value::Empty;
            }
        };
        file.into()
    }
}
#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::{
        errors::FindItError,
        expr::read_expr,
        file_wrapper::FileWrapper,
        value::{Value, ValueType},
    };

    #[test]
    fn unsupported_unary_operator() {
        let err = read_expr("-100").err();
        assert!(err.is_some())
    }

    #[test]
    fn unsupported_not_number() {
        let err = read_expr(" NOT 100").err();
        assert!(err.is_some())
    }

    #[test]
    fn not_return_empty_if_not_a_number() {
        let eval = read_expr("NOT (parent.content <> \"bad\")").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn not_expect_bool() {
        let eval = read_expr("NOT TRUE").unwrap();
        assert_eq!(eval.expected_type(), ValueType::Bool)
    }

    #[test]
    fn self_divide_number() {
        let err = read_expr(" / 100").err();
        assert!(err.is_some())
    }

    #[test]
    fn self_divide_string() -> Result<(), FindItError> {
        let file = Path::new("tests/test_cases/display/test_files/other-247.txt");
        let expr = read_expr("extension of (/ \"other-247.txt\")")?;

        let wrapper = FileWrapper::new(file.parent().unwrap().to_path_buf(), 1);

        let value = expr.eval(&wrapper);

        assert_eq!(value, Value::String("txt".into()));

        Ok(())
    }

    #[test]
    fn self_divide_path() -> Result<(), FindItError> {
        let file = Path::new("tests/test_cases/display/test_files/other-247.txt");
        let expr = read_expr("extension of (/ 'other-247.txt')")?;

        let wrapper = FileWrapper::new(file.parent().unwrap().to_path_buf(), 1);

        let value = expr.eval(&wrapper);

        assert_eq!(value, Value::String("txt".into()));

        Ok(())
    }

    #[test]
    fn self_divide_empty() -> Result<(), FindItError> {
        let file = Path::new("/no/such/file");
        let expr = read_expr("extension of (/ content)")?;

        let wrapper = FileWrapper::new(file.parent().unwrap().to_path_buf(), 1);

        let value = expr.eval(&wrapper);

        assert_eq!(value, Value::Empty);

        Ok(())
    }
}
