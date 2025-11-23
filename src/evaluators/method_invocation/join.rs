use itertools::Itertools;

use crate::{
    errors::FindItError,
    evaluators::expr::{BindingsTypes, Evaluator, EvaluatorFactory},
    file_wrapper::FileWrapper,
    parser::ast::expression::Expression,
    value::{Value, ValueType},
};

struct Join {
    target: Box<dyn Evaluator>,
    delimiter: Option<Box<dyn Evaluator>>,
}
impl Evaluator for Join {
    fn expected_type(&self) -> ValueType {
        ValueType::String
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::List(target_value) = self.target.eval(file) else {
            return Value::Empty;
        };
        let delimiter = if let Some(delim_eval) = &self.delimiter {
            let Value::String(delim_value) = delim_eval.eval(file) else {
                return Value::Empty;
            };
            delim_value
        } else {
            ",".to_string()
        };
        target_value
            .items()
            .into_iter()
            .map(|f| f.to_string())
            .join(&delimiter)
            .into()
    }
}
pub(super) fn new_join(
    target: Box<dyn Evaluator>,
    delimiter: &Option<Box<Expression>>,
    bindings: &BindingsTypes,
) -> Result<Box<dyn Evaluator>, FindItError> {
    match target.expected_type() {
        ValueType::List(_) => {
            let delimiter = match delimiter {
                Some(delim) => {
                    let delim = delim.build(bindings)?;
                    if delim.expected_type() != ValueType::String {
                        return Err(FindItError::BadExpression(
                            "Join method delimiter must be a String".to_string(),
                        ));
                    }
                    Some(delim)
                }
                None => None,
            };
            Ok(Box::new(Join { target, delimiter }))
        }
        _ => Err(FindItError::BadExpression(
            "Join method can only be applied to List type".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::{
        errors::FindItError,
        evaluators::expr::read_expr,
        file_wrapper::FileWrapper,
        value::{Value, ValueType},
    };

    #[test]
    fn test_join_no_arg() -> Result<(), FindItError> {
        let expr = read_expr("[1, 2, 4, 5].join()")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::String("1,2,4,5".into()));

        Ok(())
    }

    #[test]
    fn test_join_with_arg() -> Result<(), FindItError> {
        let expr = read_expr("[1, 2, 4, 5].join(\";\")")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::String("1;2;4;5".into()));

        Ok(())
    }

    #[test]
    fn join_return_type() -> Result<(), FindItError> {
        let expr = read_expr("[1, 2, 4, 5].join(\";\")")?;

        assert_eq!(expr.expected_type(), ValueType::String);

        Ok(())
    }

    #[test]
    fn test_join_no_target() -> Result<(), FindItError> {
        let expr = read_expr("files.join()")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn test_join_with_empty_arg() -> Result<(), FindItError> {
        let expr = read_expr("[1, 2, 4, 5].join(content)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn join_no_list() {
        let err = read_expr("123.join(\"a\")").err();
        assert!(err.is_some())
    }

    #[test]
    fn empty_join_no_list() {
        let err = read_expr("123.join()").err();
        assert!(err.is_some())
    }

    #[test]
    fn join_no_string() {
        let err = read_expr("[1, 2, 3].join(123)").err();
        assert!(err.is_some())
    }
}
