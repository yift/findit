use crate::{
    errors::FindItError,
    expr::{Evaluator, get_eval},
    file_wrapper::FileWrapper,
    parser::is_check::{IsCheck, IsType},
    unary_operators::make_negate,
    value::{Value, ValueType},
};

struct IsTrue {
    evaluator: Box<dyn Evaluator>,
}
struct IsFalse {
    evaluator: Box<dyn Evaluator>,
}
struct IsNone {
    evaluator: Box<dyn Evaluator>,
}
struct IsSome {
    evaluator: Box<dyn Evaluator>,
}

impl Evaluator for IsTrue {
    fn eval(&self, file: &FileWrapper) -> Value {
        (self.evaluator.eval(file) == Value::Bool(true)).into()
    }

    fn expected_type(&self) -> ValueType {
        ValueType::Bool
    }
}
impl Evaluator for IsFalse {
    fn eval(&self, file: &FileWrapper) -> Value {
        (self.evaluator.eval(file) == Value::Bool(false)).into()
    }

    fn expected_type(&self) -> ValueType {
        ValueType::Bool
    }
}
impl Evaluator for IsNone {
    fn eval(&self, file: &FileWrapper) -> Value {
        (self.evaluator.eval(file) == Value::Empty).into()
    }

    fn expected_type(&self) -> ValueType {
        ValueType::Bool
    }
}
impl Evaluator for IsSome {
    fn eval(&self, file: &FileWrapper) -> Value {
        (self.evaluator.eval(file) != Value::Empty).into()
    }

    fn expected_type(&self) -> ValueType {
        ValueType::Bool
    }
}

pub(crate) fn new_is_check(is_check: &IsCheck) -> Result<Box<dyn Evaluator>, FindItError> {
    let evaluator = get_eval(&is_check.expression)?;
    let checker: Box<dyn Evaluator> = match is_check.check_type {
        IsType::True => {
            if evaluator.expected_type() != ValueType::Bool {
                return Err(FindItError::BadExpression(
                    "IS (NOT) TRUE/FALSE must refer to a Boolean".into(),
                ));
            }
            Box::new(IsTrue { evaluator })
        }
        IsType::False => {
            if evaluator.expected_type() != ValueType::Bool {
                return Err(FindItError::BadExpression(
                    "IS (NOT) TRUE/FALSE must refer to a Boolean".into(),
                ));
            }
            Box::new(IsFalse { evaluator })
        }
        IsType::None => Box::new(IsNone { evaluator }),
        IsType::Some => Box::new(IsSome { evaluator }),
    };
    let checker = if is_check.negate {
        make_negate(checker)
    } else {
        checker
    };
    Ok(checker)
}
