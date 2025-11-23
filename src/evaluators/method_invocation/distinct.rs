use std::rc::Rc;

use itertools::Itertools;

use crate::{
    errors::FindItError,
    evaluators::expr::{BindingsTypes, Evaluator},
    file_wrapper::FileWrapper,
    parser::ast::methods::LambdaFunction,
    value::{List, Value, ValueType},
};

struct Distinct {
    target: Box<dyn Evaluator>,
    item_type: Rc<ValueType>,
}
impl Evaluator for Distinct {
    fn expected_type(&self) -> ValueType {
        ValueType::List(self.item_type.clone())
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::List(value) = self.target.eval(file) else {
            return Value::Empty;
        };
        let items = value.items().into_iter().unique();
        Value::List(List::new_lazy(self.item_type.clone(), items))
    }
}

struct DistinctBy {
    target: Box<dyn Evaluator>,
    lambda: Rc<Box<dyn Evaluator>>,
    items_type: Rc<ValueType>,
}
impl Evaluator for DistinctBy {
    fn expected_type(&self) -> ValueType {
        ValueType::List(self.items_type.clone())
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::List(value) = self.target.eval(file) else {
            return Value::Empty;
        };
        let file = file.clone();
        let lambda = self.lambda.clone();
        let items = value.items().into_iter().unique_by(move |val| {
            let file = file.with_binding(val.clone());
            lambda.eval(&file)
        });
        Value::List(List::new_lazy(self.items_type.clone(), items))
    }
}
pub(super) fn new_distinct(target: Box<dyn Evaluator>) -> Result<Box<dyn Evaluator>, FindItError> {
    let ValueType::List(item_type) = target.expected_type() else {
        return Err(FindItError::BadExpression(
            "Distinct method can only be applied to a List type".to_string(),
        ));
    };
    Ok(Box::new(Distinct {
        target,
        item_type: item_type.clone(),
    }))
}

pub(super) fn new_distinct_by(
    target: Box<dyn Evaluator>,
    lambda: &LambdaFunction,
    bindings: &BindingsTypes,
) -> Result<Box<dyn Evaluator>, FindItError> {
    let ValueType::List(items_type) = target.expected_type() else {
        return Err(FindItError::BadExpression(
            "Distinct by method can only be applied to a List type".to_string(),
        ));
    };
    let items_type = items_type.clone();
    let lambda = lambda.build(bindings, &items_type)?;
    Ok(Box::new(DistinctBy {
        target,
        lambda: Rc::new(lambda),
        items_type: items_type.clone(),
    }))
}

#[cfg(test)]
mod tests {
    use std::{
        path::{Path, PathBuf},
        rc::Rc,
    };

    use crate::{
        errors::FindItError,
        evaluators::expr::read_expr,
        file_wrapper::FileWrapper,
        value::{List, Value, ValueType},
    };
    #[test]
    fn test_simple_distinct() -> Result<(), FindItError> {
        let expr = read_expr("[1, 10, 10, 1, 2].distinct()")?;
        let file = &FileWrapper::new(PathBuf::new(), 1);

        assert_eq!(
            expr.eval(file),
            Value::List(List::new_eager(
                Rc::new(ValueType::Number),
                vec![Value::Number(1), Value::Number(10), Value::Number(2)].into_iter(),
            ))
        );

        Ok(())
    }

    #[test]
    fn test_simple_distinct_by() -> Result<(), FindItError> {
        let expr = read_expr("[\"abcd\", \"1234\", \"z\", \"-\"].distinctBy($str $str.len())")?;
        let file = &FileWrapper::new(PathBuf::new(), 1);

        assert_eq!(
            expr.eval(file),
            Value::List(List::new_eager(
                Rc::new(ValueType::String),
                vec![Value::String("abcd".into()), Value::String("z".into()),].into_iter(),
            ))
        );

        Ok(())
    }

    #[test]
    fn test_distinct_expected_type() -> Result<(), FindItError> {
        let expr = read_expr("[1, 2, 3, 4, 5, 6].distinct()")?;

        assert_eq!(
            expr.expected_type(),
            ValueType::List(Rc::new(ValueType::Number))
        );

        Ok(())
    }

    #[test]
    fn test_distinct_nop_return_empty() -> Result<(), FindItError> {
        let expr = read_expr("files.map($f $f.length()).distinct()")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn test_distinct_by_expected_type() -> Result<(), FindItError> {
        let expr = read_expr("[1, 2, 3, 4, 5, 6].distinctBy($f $f)")?;

        assert_eq!(
            expr.expected_type(),
            ValueType::List(Rc::new(ValueType::Number))
        );

        Ok(())
    }

    #[test]
    fn test_distinct_by_nop_return_empty() -> Result<(), FindItError> {
        let expr = read_expr("files.distinct_by($f $f.length())")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn no_list_distinct() {
        let err = read_expr("12.distinct()").err();
        assert!(err.is_some())
    }

    #[test]
    fn no_list_distinct_by() {
        let err = read_expr("12.distinct_by($f $f)").err();
        assert!(err.is_some())
    }
}
