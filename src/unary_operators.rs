use sqlparser::ast::{Expr, UnaryOperator};

use crate::{
    errors::FindItError,
    expr::{Evaluator, get_eval},
    file_wrapper::FileWrapper,
    value::{Value, ValueType},
};

pub(crate) fn new_unary_operator(
    expr: &Expr,
    operator: &UnaryOperator,
) -> Result<Box<dyn Evaluator>, FindItError> {
    let expr = get_eval(expr)?;
    match operator {
        UnaryOperator::Not => {
            if expr.expected_type() != ValueType::Bool {
                Err(FindItError::BadExpression(
                    "NOT can only applied to boolean expressions".into(),
                ))
            } else {
                Ok(Box::new(Negate { expr }))
            }
        }

        _ => Err(FindItError::BadExpression(format!(
            "Unsupported operator: {}",
            operator
        ))),
    }
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
        let eval = read_expr("NOT (parent.content <> 'bad')").unwrap();
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
