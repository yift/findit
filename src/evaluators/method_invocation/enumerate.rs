use std::rc::Rc;

use crate::{
    class_type::{Class, ClassType},
    errors::FindItError,
    evaluators::expr::Evaluator,
    file_wrapper::FileWrapper,
    value::{List, Value, ValueType},
};

const INDEX_FIELD_NAME: &str = "index";
const ITEM_FIELD_NAME: &str = "item";

struct Enumerate {
    target: Box<dyn Evaluator>,
    class_type: Rc<ValueType>,
    class_internal_type: Rc<ClassType>,
}

impl Evaluator for Enumerate {
    fn expected_type(&self) -> ValueType {
        ValueType::List(self.class_type.clone())
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::List(value) = self.target.eval(file) else {
            return Value::Empty;
        };
        let class_internal_type = self.class_internal_type.clone();
        let lst = List::new_lazy(
            self.class_type.clone(),
            value.items().into_iter().enumerate().map(move |(i, item)| {
                Value::Class(Class::new(
                    &class_internal_type,
                    vec![Value::Number(i as u64), item],
                ))
            }),
        );
        Value::List(lst)
    }
}

pub(super) fn new_enumerate(target: Box<dyn Evaluator>) -> Result<Box<dyn Evaluator>, FindItError> {
    let ValueType::List(item_type) = target.expected_type() else {
        return Err(FindItError::BadExpression(
            "Enumerate method can only be applied to List type".to_string(),
        ));
    };
    let class_internal_type = Rc::new(ClassType::new(&[
        (INDEX_FIELD_NAME.to_string(), ValueType::Number),
        (ITEM_FIELD_NAME.to_string(), (*item_type).clone()),
    ]));
    let class_type = Rc::new(ValueType::Class(class_internal_type.clone()));

    Ok(Box::new(Enumerate {
        target,
        class_type,
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
    fn test_happy_path() -> Result<(), FindItError> {
        let expr = read_expr(
            "[1, 10, 2, 20].enumerate().filter($i $i::index % 2 == 0).map($i $i::item) as Text",
        )?;
        let path = Path::new("/no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::String("[1, 2]".into()));

        Ok(())
    }

    #[test]
    fn test_empty_return_when_not_a_list() -> Result<(), FindItError> {
        let expr = read_expr("files.enumerate()")?;
        let path = Path::new("./no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        let value = expr.eval(file);

        assert_eq!(value, Value::Empty);

        Ok(())
    }

    #[test]
    fn no_list_return_error() -> Result<(), FindItError> {
        let err = read_expr("extension.enumerate()").err();

        assert!(err.is_some());

        Ok(())
    }
}
