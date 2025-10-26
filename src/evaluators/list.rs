use std::rc::Rc;

use crate::{
    errors::FindItError,
    evaluators::expr::{BindingsTypes, Evaluator, EvaluatorFactory},
    file_wrapper::FileWrapper,
    parser::ast::list::List as ListExpression,
    value::{List, Value, ValueType},
};

struct ListEval {
    items: Vec<Box<dyn Evaluator>>,
    items_type: Rc<ValueType>,
}
impl Evaluator for ListEval {
    fn expected_type(&self) -> ValueType {
        ValueType::List(self.items_type.clone())
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let items = self.items.iter().map(|f| f.eval(file));
        let list = List::new_eager(self.items_type.clone(), items);
        Value::List(list)
    }
}

impl EvaluatorFactory for ListExpression {
    fn build(&self, bindings: &BindingsTypes) -> Result<Box<dyn Evaluator>, FindItError> {
        let mut items = vec![];
        let mut items_type = None;
        for item in &self.items {
            let item = item.build(bindings)?;
            if let Some(list_item_type) = &items_type {
                if list_item_type != &item.expected_type() {
                    return Err(FindItError::BadExpression(
                        "All the items in a list must have the same type".into(),
                    ));
                }
            } else {
                items_type = Some(item.expected_type());
            }
            items.push(item);
        }
        let items_type = items_type.unwrap_or(ValueType::Empty);
        let items_type = Rc::new(items_type);
        Ok(Box::new(ListEval { items, items_type }))
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, rc::Rc};

    use crate::{
        errors::FindItError,
        evaluators::expr::read_expr,
        file_wrapper::FileWrapper,
        value::{List, Value, ValueType},
    };

    #[test]
    fn test_simple_list() -> Result<(), FindItError> {
        let expr = read_expr(":[10, 20, 30]")?;
        let file = &FileWrapper::new(PathBuf::new(), 1);

        assert_eq!(
            expr.eval(file),
            Value::List(List::new_lazy(
                Rc::new(ValueType::Number),
                [10, 20, 30].iter().map(|i| Value::Number(*i))
            ))
        );

        Ok(())
    }

    #[test]
    fn test_empty_list() -> Result<(), FindItError> {
        let expr = read_expr(":[]")?;
        let file = &FileWrapper::new(PathBuf::new(), 1);

        assert_eq!(
            expr.eval(file),
            Value::List(List::new_lazy(
                Rc::new(ValueType::Empty),
                [].iter().map(|i| Value::Number(*i))
            ))
        );

        Ok(())
    }

    #[test]
    fn test_two_types_list() {
        let err = read_expr(":[10, 20, name]").err();

        assert!(err.is_some());
    }

    #[test]
    fn test_list_expected_type() -> Result<(), FindItError> {
        let expr = read_expr(":[10, 20, 30]")?;

        assert_eq!(
            expr.expected_type(),
            ValueType::List(Rc::new(ValueType::Number))
        );

        Ok(())
    }
}
