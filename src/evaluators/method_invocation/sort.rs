use std::rc::Rc;

use itertools::Itertools;

use crate::{
    errors::FindItError,
    evaluators::expr::{BindingsTypes, Evaluator},
    file_wrapper::FileWrapper,
    parser::ast::methods::LambdaFunction,
    value::{List, Value, ValueType},
};

struct Sort {
    target: Box<dyn Evaluator>,
    item_type: Rc<ValueType>,
}
impl Evaluator for Sort {
    fn expected_type(&self) -> ValueType {
        ValueType::List(self.item_type.clone())
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::List(value) = self.target.eval(file) else {
            return Value::Empty;
        };
        let items = value.items().into_iter().sorted();
        Value::List(List::new_eager(self.item_type.clone(), items))
    }
}

struct SortBy {
    target: Box<dyn Evaluator>,
    lambda: Rc<Box<dyn Evaluator>>,
    items_type: Rc<ValueType>,
}
impl Evaluator for SortBy {
    fn expected_type(&self) -> ValueType {
        ValueType::List(self.items_type.clone())
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::List(value) = self.target.eval(file) else {
            return Value::Empty;
        };
        let items = value.items().into_iter().sorted_by(|a, b| {
            let file_a = file.with_binding(a.clone());
            let file_b = file.with_binding(b.clone());
            let key_a = self.lambda.eval(&file_a);
            let key_b = self.lambda.eval(&file_b);
            key_a.cmp(&key_b)
        });
        Value::List(List::new_eager(self.items_type.clone(), items))
    }
}
pub(super) fn new_sort(target: Box<dyn Evaluator>) -> Result<Box<dyn Evaluator>, FindItError> {
    let ValueType::List(item_type) = target.expected_type() else {
        return Err(FindItError::BadExpression(
            "Sort method can only be applied to a List type".to_string(),
        ));
    };
    Ok(Box::new(Sort {
        target,
        item_type: item_type.clone(),
    }))
}

pub(super) fn new_sort_by(
    target: Box<dyn Evaluator>,
    lambda: &LambdaFunction,
    bindings: &BindingsTypes,
) -> Result<Box<dyn Evaluator>, FindItError> {
    let ValueType::List(items_type) = target.expected_type() else {
        return Err(FindItError::BadExpression(
            "Sort by method can only be applied to a List type".to_string(),
        ));
    };
    let items_type = items_type.clone();
    let lambda = lambda.build(bindings, &items_type)?;
    Ok(Box::new(SortBy {
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
    fn test_simple_sort() -> Result<(), FindItError> {
        let expr = read_expr("[1, 14, 10].sort()")?;
        let file = &FileWrapper::new(PathBuf::new(), 1);

        assert_eq!(
            expr.eval(file),
            Value::List(List::new_eager(
                Rc::new(ValueType::Number),
                vec![Value::Number(1), Value::Number(10), Value::Number(14)].into_iter(),
            ))
        );

        Ok(())
    }

    #[test]
    fn test_simple_sort_by() -> Result<(), FindItError> {
        let expr = read_expr("[\"abcd\", \"gq\", \"z\", \"12345\"].sortBy($str $str.len())")?;
        let file = &FileWrapper::new(PathBuf::new(), 1);

        assert_eq!(
            expr.eval(file),
            Value::List(List::new_eager(
                Rc::new(ValueType::String),
                vec![
                    Value::String("z".into()),
                    Value::String("gq".into()),
                    Value::String("abcd".into()),
                    Value::String("12345".into())
                ]
                .into_iter(),
            ))
        );

        Ok(())
    }

    #[test]
    fn test_sort_expected_type() -> Result<(), FindItError> {
        let expr = read_expr("[1, 2, 3, 4, 5, 6].sort()")?;

        assert_eq!(
            expr.expected_type(),
            ValueType::List(Rc::new(ValueType::Number))
        );

        Ok(())
    }

    #[test]
    fn test_sort_nop_return_empty() -> Result<(), FindItError> {
        let expr = read_expr("files.map($f $f.length()).sort()")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn test_sort_by_expected_type() -> Result<(), FindItError> {
        let expr = read_expr("[1, 2, 3, 4, 5, 6].sort_by($f $f)")?;

        assert_eq!(
            expr.expected_type(),
            ValueType::List(Rc::new(ValueType::Number))
        );

        Ok(())
    }

    #[test]
    fn test_sort_by_nop_return_empty() -> Result<(), FindItError> {
        let expr = read_expr("files.sort_by($f $f.length())")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn length_no_list_sort() {
        let err = read_expr("12.sort()").err();
        assert!(err.is_some())
    }

    #[test]
    fn length_no_list_sort_by() {
        let err = read_expr("12.sort_by($f $f)").err();
        assert!(err.is_some())
    }
}
