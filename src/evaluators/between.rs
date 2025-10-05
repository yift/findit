use crate::errors::FindItError;
use crate::evaluators::expr::{BindingsTypes, Evaluator, EvaluatorFactory};
use crate::file_wrapper::FileWrapper;
use crate::parser::ast::between::Between as BetweenExpression;
use crate::value::{Value, ValueType};

struct Between {
    evaluator: Box<dyn Evaluator>,
    low: Box<dyn Evaluator>,
    high: Box<dyn Evaluator>,
}
impl EvaluatorFactory for BetweenExpression {
    fn build(&self, bindings: &BindingsTypes) -> Result<Box<dyn Evaluator>, FindItError> {
        let evaluator = self.reference.build(bindings)?;
        let low = self.lower_limit.build(bindings)?;
        if evaluator.expected_type() != low.expected_type() {
            return Err(FindItError::BadExpression(
                "Between low must have the same type as the expression".into(),
            ));
        }
        let high = self.upper_limit.build(bindings)?;
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
