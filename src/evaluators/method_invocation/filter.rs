use std::rc::Rc;

use crate::{
    errors::FindItError,
    evaluators::expr::{BindingsTypes, Evaluator},
    file_wrapper::FileWrapper,
    parser::ast::methods::LambdaFunction,
    value::{List, Value, ValueType},
};

struct Filter {
    target: Box<dyn Evaluator>,
    lambda: Rc<Box<dyn Evaluator>>,
    items_type: Rc<ValueType>,
}

impl Evaluator for Filter {
    fn expected_type(&self) -> ValueType {
        ValueType::List(self.items_type.clone())
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::List(value) = self.target.eval(file) else {
            return Value::Empty;
        };
        let lambda = self.lambda.clone();
        let file = file.clone();
        let iter = value.items().into_iter().filter(move |item| {
            let new_file = file.with_binding(item.clone());
            lambda.eval(&new_file) == Value::Bool(true)
        });
        Value::List(List::new_lazy(self.items_type.clone(), iter))
    }
}

pub(super) fn new_filter(
    target: Box<dyn Evaluator>,
    lambda: &LambdaFunction,
    bindings: &BindingsTypes,
) -> Result<Box<dyn Evaluator>, FindItError> {
    let ValueType::List(items_type) = target.expected_type() else {
        return Err(FindItError::BadExpression(
            "Filter method can only be applied to List type".to_string(),
        ));
    };
    let items_type = items_type.clone();
    let lambda_evaluator = lambda.build(bindings, &items_type)?;
    if lambda_evaluator.expected_type() != ValueType::Bool {
        return Err(FindItError::BadExpression(
            "Filter lambda must return a Bool value".to_string(),
        ));
    }
    Ok(Box::new(Filter {
        target,
        lambda: Rc::new(lambda_evaluator),
        items_type,
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
    fn test_simple_filter() -> Result<(), FindItError> {
        let expr = read_expr(":[1, 2, 3, 4, 5, 6].filter({n} {n} % 2 == 0)")?;
        let file = &FileWrapper::new(PathBuf::new(), 1);

        assert_eq!(
            expr.eval(file),
            Value::List(List::new_eager(
                Rc::new(ValueType::Number),
                vec![Value::Number(2), Value::Number(4), Value::Number(6)].into_iter(),
            ))
        );

        Ok(())
    }

    #[test]
    fn test_filter_nop_return_empty() -> Result<(), FindItError> {
        let expr = read_expr("files.filter({f} {f}.length() % 2 == 0)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn length_no_list_filter() {
        let err = read_expr("12.filter({f} {f})").err();
        assert!(err.is_some())
    }

    #[test]
    fn length_no_bool_filter() {
        let err = read_expr(":[1 ,2, 3].filter({f} {f})").err();
        assert!(err.is_some())
    }
}
