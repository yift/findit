use crate::{
    errors::FindItError,
    evaluators::expr::{BindingsTypes, Evaluator, EvaluatorFactory},
    file_wrapper::FileWrapper,
    parser::ast::class::ClassAccess,
    value::{Value, ValueType},
};

struct Access {
    target: Box<dyn Evaluator>,
    index: usize,
    value_type: ValueType,
}

impl Evaluator for Access {
    fn expected_type(&self) -> ValueType {
        self.value_type.clone()
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::Class(cls) = self.target.eval(file) else {
            return Value::Empty;
        };
        cls.get(self.index)
    }
}
impl EvaluatorFactory for ClassAccess {
    fn build(&self, bindings: &BindingsTypes) -> Result<Box<dyn Evaluator>, FindItError> {
        let target = self.target.build(bindings)?;
        let ValueType::Class(cls) = target.expected_type() else {
            return Err(FindItError::BadExpression("Can only access classes".into()));
        };
        let (index, value_type) = cls.get_index_and_type(&self.field)?;
        Ok(Box::new(Access {
            target,
            index,
            value_type,
        }))
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
    fn simple_happy_path() -> Result<(), FindItError> {
        let expr = read_expr("{:one 1, :two \"2\"}::two")?;
        let file = Path::new("/tmp");
        let wrapper = FileWrapper::new(file.to_path_buf(), 1);

        let value = expr.eval(&wrapper);

        assert_eq!(value, Value::String("2".into()));

        Ok(())
    }

    #[test]
    fn empty_return_empty() -> Result<(), FindItError> {
        let expr = read_expr("[1].skip(3).map($a {:a $a}).first()::a ")?;
        let file = Path::new("/tmp");
        let wrapper = FileWrapper::new(file.to_path_buf(), 1);

        let value = expr.eval(&wrapper);

        assert_eq!(value, Value::Empty);

        Ok(())
    }

    #[test]
    fn return_type() -> Result<(), FindItError> {
        let expr = read_expr("{:one 1, :two \"2\"}::two")?;

        assert_eq!(expr.expected_type(), ValueType::String);

        Ok(())
    }

    #[test]
    fn access_err_for_invalid_name() -> Result<(), FindItError> {
        let err = read_expr("{:one 1, :two \"2\"}::three").err();

        assert!(err.is_some());

        Ok(())
    }

    #[test]
    fn access_not_a_class() -> Result<(), FindItError> {
        let err = read_expr("[1, 2, 3]::0").err();

        assert!(err.is_some());

        Ok(())
    }
}
