use std::rc::Rc;

use crate::{
    errors::FindItError,
    evaluators::expr::Evaluator,
    file_wrapper::FileWrapper,
    value::{List, Value, ValueType},
};

struct ReverseString {
    target: Box<dyn Evaluator>,
}
impl Evaluator for ReverseString {
    fn expected_type(&self) -> ValueType {
        ValueType::String
    }

    fn eval(&self, file: &FileWrapper) -> Value {
        let target_value = self.target.eval(file);
        match target_value {
            Value::String(s) => s.chars().rev().collect::<String>().into(),
            _ => Value::Empty,
        }
    }
}

struct ReverseList {
    target: Box<dyn Evaluator>,
    item_type: Rc<ValueType>,
}
impl Evaluator for ReverseList {
    fn expected_type(&self) -> ValueType {
        ValueType::List(self.item_type.clone())
    }

    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::List(lst) = self.target.eval(file) else {
            return Value::Empty;
        };
        let mut vec = vec![];
        for i in lst.items().into_iter() {
            vec.push(i);
        }
        vec.reverse();

        Value::List(List::new_from_vec(self.item_type.clone(), vec))
    }
}

pub(super) fn new_reverse(target: Box<dyn Evaluator>) -> Result<Box<dyn Evaluator>, FindItError> {
    match target.expected_type() {
        ValueType::String => Ok(Box::new(ReverseString { target })),
        ValueType::List(item_type) => Ok(Box::new(ReverseList { target, item_type })),
        _ => Err(FindItError::BadExpression(
            "Reverse method can only be applied to String type".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use std::{path::Path, rc::Rc};

    use crate::{
        evaluators::expr::read_expr,
        file_wrapper::FileWrapper,
        value::{List, Value, ValueType},
    };

    #[test]
    fn reverse_null_str_return_empty() {
        let eval = read_expr("content.REVERSE()").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn reverse_expect_string() {
        let expr = read_expr("\"\".REVERSE()").unwrap();
        assert_eq!(expr.expected_type(), ValueType::String);
    }

    #[test]
    fn reverse_expect_list() {
        let expr = read_expr("[1].REVERSE()").unwrap();
        assert_eq!(
            expr.expected_type(),
            ValueType::List(Rc::new(ValueType::Number))
        );
    }

    #[test]
    fn reverse_works() {
        let eval = read_expr("\"123\".REVERSE()").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::String("321".into()))
    }

    #[test]
    fn length_no_string_reverse() {
        let err = read_expr("12.reverse()").err();
        assert!(err.is_some())
    }

    #[test]
    fn reverse_list_works() {
        let eval = read_expr("[1, 2, 3].reverse()").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);

        assert_eq!(
            value,
            Value::List(List::new_eager(
                Rc::new(ValueType::Number),
                vec![Value::Number(3), Value::Number(2), Value::Number(1)].into_iter(),
            ))
        );
    }

    #[test]
    fn reverse_list_returns_empty_for_empty() {
        let eval = read_expr("me.words().reverse()").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Empty,);
    }
}
