use std::collections::VecDeque;

use rand::RngCore;

use crate::{
    errors::FindItError,
    evaluators::expr::Evaluator,
    file_wrapper::FileWrapper,
    value::{Value, ValueType},
};

struct Random {}
impl Evaluator for Random {
    fn expected_type(&self) -> ValueType {
        ValueType::Number
    }
    fn eval(&self, _: &FileWrapper) -> Value {
        let mut rng = rand::rng();
        Value::Number(rng.next_u64())
    }
}

pub(crate) fn build_rand(
    args: VecDeque<Box<dyn Evaluator>>,
) -> Result<Box<dyn Evaluator>, FindItError> {
    if !args.is_empty() {
        return Err(FindItError::BadExpression("RANDOM with arguments.".into()));
    }

    Ok(Box::new(Random {}))
}

#[cfg(test)]
mod tests {
    use std::env;

    use crate::{
        errors::FindItError,
        evaluators::expr::read_expr,
        file_wrapper::FileWrapper,
        value::{Value, ValueType},
    };

    #[test]
    fn rand_with_args() {
        let sql = "RAND(1, 2)";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }

    #[test]
    fn rand_expected_value() {
        let sql = "RAND()";
        let eval = read_expr(sql).unwrap();

        assert_eq!(eval.expected_type(), ValueType::Number);
    }

    #[test]
    fn rand_execution() -> Result<(), FindItError> {
        let sql = "RANDOM()";
        let eval = read_expr(sql).unwrap();

        let file = env::current_dir()?;
        let wrapper = FileWrapper::new(file, 1);

        let Value::Number(_) = eval.eval(&wrapper) else {
            panic!("Not a number!")
        };

        Ok(())
    }
}
