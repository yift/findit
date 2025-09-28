use std::{collections::VecDeque, env};

use crate::{
    errors::FindItError,
    evaluators::expr::Evaluator,
    file_wrapper::FileWrapper,
    value::{Value, ValueType},
};

struct Env {
    name: Box<dyn Evaluator>,
}
impl Evaluator for Env {
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::String(name) = self.name.eval(file) else {
            return Value::Empty;
        };

        let Ok(value) = env::var(name) else {
            return Value::Empty;
        };
        Value::String(value)
    }
    fn expected_type(&self) -> ValueType {
        ValueType::String
    }
}
pub(crate) fn build_env(
    mut arguments: VecDeque<Box<dyn Evaluator>>,
) -> Result<Box<dyn Evaluator>, FindItError> {
    if arguments.len() > 1 {
        return Err(FindItError::BadExpression(
            "env can not handle more than one argument.".into(),
        ));
    }
    let Some(name) = arguments.pop_front() else {
        return Err(FindItError::BadExpression(
            "env must have one argument.".into(),
        ));
    };
    if name.expected_type() != ValueType::String {
        return Err(FindItError::BadExpression(
            "env must argument must be a string.".into(),
        ));
    }
    Ok(Box::new(Env { name }))
}

#[cfg(test)]
mod tests {

    use std::{env, path::Path};

    use crate::{
        errors::FindItError,
        evaluators::expr::read_expr,
        file_wrapper::FileWrapper,
        value::{Value, ValueType},
    };

    #[test]
    fn env_with_no_args() {
        let sql = "env()";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }

    #[test]
    fn env_with_two_args() {
        let sql = "env(\"one\", \"two\")";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }

    #[test]
    fn env_with_numeric_args() {
        let sql = "env(1)";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }

    #[test]
    fn env_return_the_correct_value() -> Result<(), FindItError> {
        let sql = "env(\"USER\")";
        let expr = read_expr(sql)?;
        let expected = env::var("USER").unwrap();

        let file = FileWrapper::new(Path::new("/no").to_path_buf(), 1);

        assert_eq!(expr.eval(&file), Value::String(expected));

        Ok(())
    }

    #[test]
    fn env_expected_type() -> Result<(), FindItError> {
        let sql = "env(\"USER\")";
        let expr = read_expr(sql)?;

        assert_eq!(expr.expected_type(), ValueType::String);

        Ok(())
    }

    #[test]
    fn env_return_nothing_for_nothing() -> Result<(), FindItError> {
        let sql = "env(content)";
        let expr = read_expr(sql)?;

        let file = FileWrapper::new(Path::new("/no").to_path_buf(), 1);

        assert_eq!(expr.eval(&file), Value::Empty);

        Ok(())
    }

    #[test]
    fn env_return_nothing_for_unknown_variable() -> Result<(), FindItError> {
        let sql = "env(\"NO_SUCH_THING\")";
        let expr = read_expr(sql)?;

        let file = FileWrapper::new(Path::new("/no").to_path_buf(), 1);

        assert_eq!(expr.eval(&file), Value::Empty);

        Ok(())
    }
}
