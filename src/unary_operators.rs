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
}
