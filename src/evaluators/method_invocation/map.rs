use std::rc::Rc;

use crate::{
    errors::FindItError,
    evaluators::expr::{BindingsTypes, Evaluator},
    file_wrapper::FileWrapper,
    parser::ast::methods::LambdaFunction,
    value::{List, Value, ValueType},
};

struct Map {
    target: Box<dyn Evaluator>,
    lambda: Rc<Box<dyn Evaluator>>,
    items_type: Rc<ValueType>,
}

impl Evaluator for Map {
    fn expected_type(&self) -> ValueType {
        ValueType::List(self.items_type.clone())
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::List(value) = self.target.eval(file) else {
            return Value::Empty;
        };
        let lambda = self.lambda.clone();
        let file = file.clone();
        let iter = value.items().into_iter().map(move |item| {
            let new_file = file.with_binding(item);
            lambda.eval(&new_file)
        });
        Value::List(List::new_lazy(self.items_type.clone(), iter))
    }
}

pub(super) fn new_map(
    target: Box<dyn Evaluator>,
    lambda: &LambdaFunction,
    bindings: &BindingsTypes,
) -> Result<Box<dyn Evaluator>, FindItError> {
    let ValueType::List(input_item_type) = target.expected_type() else {
        return Err(FindItError::BadExpression(
            "Map method can only be applied to List type".to_string(),
        ));
    };
    let lambda = lambda.build(bindings, &input_item_type)?;
    let output_item_type = lambda.expected_type().clone();
    let lambda = Rc::new(lambda);

    Ok(Box::new(Map {
        target,
        lambda,
        items_type: Rc::new(output_item_type),
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
    fn test_simple_map() -> Result<(), FindItError> {
        let expr = read_expr(":[1, 2, 3].map({n} {n} * 10)")?;
        let file = &FileWrapper::new(PathBuf::new(), 1);

        assert_eq!(
            expr.eval(file),
            Value::List(List::new_eager(
                Rc::new(ValueType::Number),
                vec![Value::Number(10), Value::Number(20), Value::Number(30)].into_iter(),
            ))
        );

        Ok(())
    }

    #[test]
    fn test_map_nop_return_empty() -> Result<(), FindItError> {
        let expr = read_expr("files.map({f} {f}.content)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn test_filter_map_sum() -> Result<(), FindItError> {
        let expr =
            read_expr(":[1, 2, 3, 4, 5, 6].filter({n} {n} % 2 == 0).map({n} {n} * 10).sum()")?;
        let file = &FileWrapper::new(PathBuf::new(), 1);

        assert_eq!(expr.eval(file), Value::Number(120),);

        Ok(())
    }

    #[test]
    fn length_no_list_map() {
        let err = read_expr("12.map({f} {f})").err();
        assert!(err.is_some())
    }
}
