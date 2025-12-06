use std::{collections::HashMap, rc::Rc};

use crate::{
    class_type::{Class, ClassType},
    errors::FindItError,
    evaluators::expr::{BindingsTypes, Evaluator},
    file_wrapper::FileWrapper,
    parser::ast::methods::LambdaFunction,
    value::{List, Value, ValueType},
};

const KEY_FIELD_NAME: &str = "key";
const VALUES_FIELD_NAME: &str = "values";

struct GroupBy {
    target: Box<dyn Evaluator>,
    lambda: Rc<Box<dyn Evaluator>>,
    class_type: Rc<ValueType>,
    item_type: Rc<ValueType>,
    class_internal_type: Rc<ClassType>,
}

impl Evaluator for GroupBy {
    fn expected_type(&self) -> ValueType {
        ValueType::List(self.class_type.clone())
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::List(value) = self.target.eval(file) else {
            return Value::Empty;
        };
        // This should work, the key is "muteable" only when a list move from lazy to eager, which should work well for hash and equals.
        #[allow(clippy::mutable_key_type)]
        let mut groups = HashMap::new();
        for item in value.items() {
            let new_file = file.with_binding(item.clone());
            let key = self.lambda.eval(&new_file);
            groups.entry(key).or_insert_with(Vec::new).push(item);
        }
        let lst = List::new_eager(
            self.class_type.clone(),
            groups.into_iter().map(|(key, val)| {
                Value::Class(Class::new(
                    &self.class_internal_type,
                    vec![
                        key,
                        Value::List(List::new_eager(self.item_type.clone(), val.into_iter())),
                    ],
                ))
            }),
        );
        Value::List(lst)
    }
}

pub(super) fn new_group_by(
    target: Box<dyn Evaluator>,
    lambda: &LambdaFunction,
    bindings: &BindingsTypes,
) -> Result<Box<dyn Evaluator>, FindItError> {
    let ValueType::List(item_type) = target.expected_type() else {
        return Err(FindItError::BadExpression(
            "Map method can only be applied to List type".to_string(),
        ));
    };
    let lambda = lambda.build(bindings, &item_type)?;
    let key_type = lambda.expected_type().clone();
    let lambda = Rc::new(lambda);
    let class_internal_type = Rc::new(ClassType::new(&[
        (KEY_FIELD_NAME.to_string(), key_type),
        (
            VALUES_FIELD_NAME.to_string(),
            ValueType::List(item_type.clone()),
        ),
    ]));
    let class_type = Rc::new(ValueType::Class(class_internal_type.clone()));

    Ok(Box::new(GroupBy {
        target,
        lambda,
        class_type,
        item_type,
        class_internal_type,
    }))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::{
        errors::FindItError, evaluators::expr::read_expr, file_wrapper::FileWrapper, value::Value,
    };

    #[test]
    fn test_happy_complex_path() -> Result<(), FindItError> {
        let expr = read_expr(
            "files.groupBy($f $f.extension).map($g {:extension $g::key, :count $g::values.length()}).sortBy($g $g::extension) as Text",
        )?;
        let path = Path::new("./tests/test_cases/order_by/test_files/next/emma/amelia");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::String("[{\"extension\":bash, \"count\":1}, {\"extension\":json, \"count\":1}, {\"extension\":txt, \"count\":5}]".into()));

        Ok(())
    }

    #[test]
    fn test_happy_simple_path() -> Result<(), FindItError> {
        let expr = read_expr("[1, 2, 3, 4, 5, 6].groupBy($x $x % 2).sortBy($g $g::key)")?;
        let path = Path::new("./tests/test_cases/order_by/test_files/next/emma/amelia");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        let value = expr.eval(file);

        let expected =
            read_expr("[{:key 0, :values [2,4,6]},{:key 1, :values [1,3,5]}]")?.eval(file);
        assert_eq!(value, expected);

        Ok(())
    }

    #[test]
    fn test_empty_return_when_not_a_list() -> Result<(), FindItError> {
        let expr = read_expr("files.groupBy($x $x.extension)")?;
        let path = Path::new("./no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        let value = expr.eval(file);

        assert_eq!(value, Value::Empty);

        Ok(())
    }

    #[test]
    fn no_list_return_error() -> Result<(), FindItError> {
        let err = read_expr("extension.groupBy($x $x.extension)").err();

        assert!(err.is_some());

        Ok(())
    }
}
