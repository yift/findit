use crate::{
    errors::FindItError,
    expr::{Evaluator, get_eval},
    file_wrapper::FileWrapper,
    parser::ast::is_check::{IsCheck, IsType},
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

#[cfg(test)]
mod tests {

    use std::path::Path;

    use crate::{
        errors::FindItError,
        expr::read_expr,
        file_wrapper::FileWrapper,
        value::{Value, ValueType},
    };

    #[test]
    fn test_is_true_with_non_bool_returns_error() {
        let err = read_expr("20 IS TRUE").err();

        assert!(err.is_some());
    }

    #[test]
    fn test_is_false_with_non_bool_returns_error() {
        let err = read_expr("'test' IS FALSE").err();

        assert!(err.is_some());
    }

    fn test_expected_type(name: &str) -> Result<(), FindItError> {
        let expr = read_expr(&format!("TRUE {}", name))?;
        let tp = expr.expected_type();

        assert_eq!(tp, ValueType::Bool);

        Ok(())
    }

    #[test]
    fn is_false_expected_type() -> Result<(), FindItError> {
        test_expected_type("IS FALSE")
    }

    #[test]
    fn is_true_expected_type() -> Result<(), FindItError> {
        test_expected_type("IS TRUE")
    }

    #[test]
    fn is_some_expected_type() -> Result<(), FindItError> {
        test_expected_type("IS some")
    }

    #[test]
    fn is_none_expected_type() -> Result<(), FindItError> {
        test_expected_type("is none")
    }

    #[test]
    fn test_is_some_true() -> Result<(), FindItError> {
        let expr = read_expr("content is some")?;

        let file = Path::new("tests/test_cases/display/test_files/other-247.txt");
        let wrapper = FileWrapper::new(file.to_path_buf(), 1);

        let value = expr.eval(&wrapper);

        assert_eq!(value, Value::Bool(true));

        Ok(())
    }

    #[test]
    fn test_is_some_false() -> Result<(), FindItError> {
        let expr = read_expr("content is some")?;

        let file = Path::new("/no/such/file");
        let wrapper = FileWrapper::new(file.to_path_buf(), 1);

        let value = expr.eval(&wrapper);

        assert_eq!(value, Value::Bool(false));

        Ok(())
    }

    #[test]
    fn test_is_none_false() -> Result<(), FindItError> {
        let expr = read_expr("content is none")?;

        let file = Path::new("tests/test_cases/display/test_files/other-247.txt");
        let wrapper = FileWrapper::new(file.to_path_buf(), 1);

        let value = expr.eval(&wrapper);

        assert_eq!(value, Value::Bool(false));

        Ok(())
    }

    #[test]
    fn test_is_none_true() -> Result<(), FindItError> {
        let expr = read_expr("(content of self) is none")?;

        let file = Path::new("/no/such/file");
        let wrapper = FileWrapper::new(file.to_path_buf(), 1);

        let value = expr.eval(&wrapper);

        assert_eq!(value, Value::Bool(true));

        Ok(())
    }
}
