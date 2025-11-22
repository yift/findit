use std::rc::Rc;

use crate::{
    errors::FindItError,
    evaluators::expr::{BindingsTypes, Evaluator},
    file_wrapper::FileWrapper,
    parser::ast::methods::LambdaFunction,
    value::{Value, ValueType},
};

struct Any {
    target: Box<dyn Evaluator>,
    lambda: Rc<Box<dyn Evaluator>>,
}

impl Evaluator for Any {
    fn expected_type(&self) -> ValueType {
        ValueType::Bool
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::List(value) = self.target.eval(file) else {
            return Value::Empty;
        };
        let lambda = self.lambda.clone();
        let file = file.clone();
        value
            .items()
            .into_iter()
            .any(move |item| {
                let new_file = file.with_binding(item.clone());
                lambda.eval(&new_file) == Value::Bool(true)
            })
            .into()
    }
}

pub(super) fn new_any(
    target: Box<dyn Evaluator>,
    lambda: &LambdaFunction,
    bindings: &BindingsTypes,
) -> Result<Box<dyn Evaluator>, FindItError> {
    let ValueType::List(items_type) = target.expected_type() else {
        return Err(FindItError::BadExpression(
            "Any method can only be applied to List type".to_string(),
        ));
    };
    let lambda_evaluator = lambda.build(bindings, &items_type)?;
    if lambda_evaluator.expected_type() != ValueType::Bool {
        return Err(FindItError::BadExpression(
            "Any lambda must return a Bool value".to_string(),
        ));
    }
    Ok(Box::new(Any {
        target,
        lambda: Rc::new(lambda_evaluator),
    }))
}

#[cfg(test)]
mod tests {
    use crate::{
        errors::FindItError,
        evaluators::expr::read_expr,
        file_wrapper::FileWrapper,
        value::{Value, ValueType},
    };
    use std::path::{Path, PathBuf};

    #[test]
    fn test_simple_any_true() -> Result<(), FindItError> {
        let expr = read_expr(":[1, 2, 3, 4, 5, 6].any({n} {n} > 4)")?;
        let file = &FileWrapper::new(PathBuf::new(), 1);

        assert_eq!(expr.eval(file), Value::Bool(true));

        Ok(())
    }

    #[test]
    fn test_simple_any_false() -> Result<(), FindItError> {
        let expr = read_expr(":[1, 2, 3, 4, 5, 6].any({n} {n} > 10)")?;
        let file = &FileWrapper::new(PathBuf::new(), 1);

        assert_eq!(expr.eval(file), Value::Bool(false));

        Ok(())
    }

    #[test]
    fn test_any_nop_return_empty() -> Result<(), FindItError> {
        let expr = read_expr("files.any({f} {f}.length() % 2 == 0)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn length_no_list_any() {
        let err = read_expr("12.any({f} {f})").err();
        assert!(err.is_some())
    }

    #[test]
    fn length_no_bool_any() {
        let err = read_expr(":[1 ,2, 3].any({f} {f})").err();
        assert!(err.is_some())
    }
    #[test]
    fn test_any_expected_type() -> Result<(), FindItError> {
        let expr = read_expr(":[1, 2, 3, 4, 5, 6].any({n} {n} < 20)")?;

        assert_eq!(expr.expected_type(), ValueType::Bool);

        Ok(())
    }
}
