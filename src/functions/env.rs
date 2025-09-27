use std::{collections::VecDeque, env};

use crate::{
    errors::FindItError,
    expr::Evaluator,
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
    Ok(Box::new(Env { name }))
}
