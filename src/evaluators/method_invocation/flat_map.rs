use std::rc::Rc;

use crate::{
    errors::FindItError,
    evaluators::expr::{BindingsTypes, Evaluator},
    file_wrapper::FileWrapper,
    lazy_list::LazyList,
    parser::ast::methods::LambdaFunction,
    value::{List, Value, ValueType},
};

struct FlatMap {
    target: Box<dyn Evaluator>,
    lambda: Rc<Box<dyn Evaluator>>,
    items_type: Rc<ValueType>,
}

impl Evaluator for FlatMap {
    fn expected_type(&self) -> ValueType {
        ValueType::List(self.items_type.clone())
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::List(value) = self.target.eval(file) else {
            return Value::Empty;
        };
        let lambda = self.lambda.clone();
        let file = file.clone();
        let iter = value.items().into_iter().flat_map(move |item| {
            let new_file = file.with_binding(item);
            if let Value::List(list) = lambda.eval(&new_file) {
                list.items().into_iter()
            } else {
                let items: LazyList<Value> = vec![].into();
                items.into_iter()
            }
        });
        Value::List(List::new_lazy(self.items_type.clone(), iter))
    }
}

pub(super) fn new_flat_map(
    target: Box<dyn Evaluator>,
    lambda: &LambdaFunction,
    bindings: &BindingsTypes,
) -> Result<Box<dyn Evaluator>, FindItError> {
    let ValueType::List(input_item_type) = target.expected_type() else {
        return Err(FindItError::BadExpression(
            "FlatMap method can only be applied to List type".to_string(),
        ));
    };
    let lambda = lambda.build(bindings, &input_item_type)?;
    let ValueType::List(output_item_type) = lambda.expected_type().clone() else {
        return Err(FindItError::BadExpression(
            "FlatMap lambda must return a List".to_string(),
        ));
    };
    let lambda = Rc::new(lambda);

    Ok(Box::new(FlatMap {
        target,
        lambda,
        items_type: output_item_type,
    }))
}

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::{BufRead, BufReader},
        path::{Path, PathBuf},
        rc::Rc,
    };

    use itertools::Itertools;

    use crate::{
        errors::FindItError,
        evaluators::expr::read_expr,
        file_wrapper::FileWrapper,
        value::{List, Value, ValueType},
    };

    #[test]
    fn test_simple_flat_map() -> Result<(), FindItError> {
        let expr = read_expr(":[10, 20, 30].flat_map($n :[$n+1, $n+5])")?;
        let file = &FileWrapper::new(PathBuf::new(), 1);

        assert_eq!(
            expr.eval(file),
            Value::List(List::new_eager(
                Rc::new(ValueType::Number),
                vec![
                    Value::Number(11),
                    Value::Number(15),
                    Value::Number(21),
                    Value::Number(25),
                    Value::Number(31),
                    Value::Number(35)
                ]
                .into_iter(),
            ))
        );

        Ok(())
    }
    #[test]
    fn test_flat_map_with_files() -> Result<(), FindItError> {
        let expr = read_expr("me.files.flat_map($f $f.lines()).sort()")?;
        let file = &FileWrapper::new(PathBuf::from("tests/test_cases/display/test_files/"), 1);

        let value = expr.eval(file);

        let expected = file
            .path()
            .read_dir()?
            .map(|f| f.unwrap().path())
            .filter(|f| f.is_file())
            .map(File::open)
            .map(|f| BufReader::new(f.unwrap()))
            .flat_map(|f| f.lines())
            .map(|l| l.unwrap())
            .sorted()
            .map(Value::String)
            .collect();
        assert_eq!(
            value,
            Value::List(List::new_from_vec(Rc::new(ValueType::String), expected))
        );
        Ok(())
    }

    #[test]
    fn test_flat_map_with_nothing() -> Result<(), FindItError> {
        let expr = read_expr(":[10, 20, 30].flat_map($n me.words())")?;
        let file = &FileWrapper::new(PathBuf::new(), 1);

        assert_eq!(
            expr.eval(file),
            Value::List(List::new_from_vec(Rc::new(ValueType::String), vec![]))
        );

        Ok(())
    }

    #[test]
    fn test_flat_map_nop_return_empty() -> Result<(), FindItError> {
        let expr = read_expr("files.flat_map($f $f.words())")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn no_list_flat_map() {
        let err = read_expr("12.flatMap($f :[$f])").err();
        assert!(err.is_some())
    }

    #[test]
    fn no_list_lambda_flat_map() {
        let err = read_expr(":[12].flatMap($f $f)").err();
        assert!(err.is_some())
    }
}
