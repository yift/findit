use crate::errors::FindItError;
use crate::evaluators::expr::{Evaluator, get_eval};
use crate::file_wrapper::FileWrapper;
use crate::parser::ast::between::Between as BetweenExpression;
use crate::value::{Value, ValueType};

struct Between {
    evaluator: Box<dyn Evaluator>,
    low: Box<dyn Evaluator>,
    high: Box<dyn Evaluator>,
}
impl TryFrom<&BetweenExpression> for Box<dyn Evaluator> {
    type Error = FindItError;
    fn try_from(between: &BetweenExpression) -> Result<Self, Self::Error> {
        let evaluator = get_eval(&between.reference)?;
        let low = get_eval(&between.lower_limit)?;
        if evaluator.expected_type() != low.expected_type() {
            return Err(FindItError::BadExpression(
                "Between low must have the same type as the expression".into(),
            ));
        }
        let high = get_eval(&between.upper_limit)?;
        if evaluator.expected_type() != high.expected_type() {
            return Err(FindItError::BadExpression(
                "Between high must have the same type as the expression".into(),
            ));
        }
        Ok(Box::new(Between {
            evaluator,
            low,
            high,
        }))
    }
}

impl Evaluator for Between {
    fn eval(&self, file: &FileWrapper) -> Value {
        let value = self.evaluator.eval(file);
        if value == Value::Empty {
            return Value::Empty;
        }
        let low = self.low.eval(file);
        if low == Value::Empty {
            return Value::Empty;
        }
        if value < low {
            return Value::Bool(false);
        }
        let high = self.high.eval(file);
        if high == Value::Empty {
            return Value::Empty;
        }
        if value > high {
            Value::Bool(false)
        } else {
            Value::Bool(true)
        }
    }

    fn expected_type(&self) -> ValueType {
        ValueType::Bool
    }
}
