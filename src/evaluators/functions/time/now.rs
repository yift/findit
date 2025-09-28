use std::collections::VecDeque;

use chrono::Local;

use crate::{
    errors::FindItError,
    evaluators::expr::Evaluator,
    file_wrapper::FileWrapper,
    value::{Value, ValueType},
};

struct Now {}

impl Evaluator for Now {
    fn eval(&self, _: &FileWrapper) -> Value {
        Local::now().into()
    }
    fn expected_type(&self) -> ValueType {
        ValueType::Date
    }
}
pub(crate) fn build_now(
    args: VecDeque<Box<dyn Evaluator>>,
) -> Result<Box<dyn Evaluator>, FindItError> {
    if !args.is_empty() {
        return Err(FindItError::BadExpression("NOW with arguments.".into()));
    }

    Ok(Box::new(Now {}))
}

#[cfg(test)]
mod tests {
    use std::env;

    use chrono::Local;

    use crate::{
        errors::FindItError,
        evaluators::expr::read_expr,
        file_wrapper::FileWrapper,
        value::{Value, ValueType},
    };

    #[test]
    fn now_with_args() {
        let sql = "NOW(1, 2)";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }

    #[test]
    fn now_expected_value() {
        let sql = "NOW()";
        let eval = read_expr(sql).unwrap();

        assert_eq!(eval.expected_type(), ValueType::Date);
    }

    #[test]
    fn now_execution() -> Result<(), FindItError> {
        let sql = "Now()";
        let eval = read_expr(sql)?;

        let file = env::current_dir()?;
        let wrapper = FileWrapper::new(file, 1);

        let start = Local::now();
        let Value::Date(result) = eval.eval(&wrapper) else {
            panic!("Not a number!")
        };

        assert!((result - start).num_seconds() < 5);

        Ok(())
    }
}
