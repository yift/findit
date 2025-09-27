use std::collections::VecDeque;

use crate::{
    errors::FindItError,
    expr::Evaluator,
    file_wrapper::FileWrapper,
    value::{Value, ValueType},
};

struct Coalesce {
    args: VecDeque<Box<dyn Evaluator>>,
    value_type: ValueType,
}

impl Evaluator for Coalesce {
    fn eval(&self, file: &FileWrapper) -> Value {
        for arg in &self.args {
            let value = arg.eval(file);
            if value != Value::Empty {
                return value;
            }
        }
        Value::Empty
    }
    fn expected_type(&self) -> ValueType {
        self.value_type
    }
}

pub(crate) fn build_coalesce(
    args: VecDeque<Box<dyn Evaluator>>,
) -> Result<Box<dyn Evaluator>, FindItError> {
    let Some(value_type) = args.iter().next().map(|e| e.expected_type()) else {
        return Err(FindItError::BadExpression(
            "coalesce must have arguments.".into(),
        ));
    };
    for a in &args {
        if a.expected_type() != value_type {
            return Err(FindItError::BadExpression(
                "All the coalesce arguments must have the same type.".into(),
            ));
        }
    }
    Ok(Box::new(Coalesce { args, value_type }))
}
