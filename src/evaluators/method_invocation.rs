use std::{ops::Deref, rc::Rc};

use itertools::Itertools;

use crate::{
    errors::FindItError,
    evaluators::{
        expr::{BindingsTypes, Evaluator, EvaluatorFactory},
        extract::MeExtractor,
    },
    file_wrapper::FileWrapper,
    parser::ast::methods::{LambdaFunction, Method, MethodInvocation},
    value::{List, Value, ValueType},
};

struct Length {
    target: Box<dyn Evaluator>,
}
impl Evaluator for Length {
    fn expected_type(&self) -> ValueType {
        ValueType::Number
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let target_value = self.target.eval(file);
        match target_value {
            Value::List(list) => list.count().into(),
            Value::String(s) => s.len().into(),
            Value::Path(f) => {
                if let Ok(metadata) = std::fs::metadata(&f)
                    && metadata.is_file()
                    && let Ok(content) = std::fs::read(&f)
                {
                    content.len().into()
                } else {
                    Value::Empty
                }
            }
            _ => Value::Empty,
        }
    }
}

struct ToUpper {
    target: Box<dyn Evaluator>,
}

impl Evaluator for ToUpper {
    fn expected_type(&self) -> ValueType {
        ValueType::String
    }

    fn eval(&self, file: &FileWrapper) -> Value {
        let target_value = self.target.eval(file);
        match target_value {
            Value::String(s) => s.to_uppercase().into(),
            _ => Value::Empty,
        }
    }
}

struct ToLower {
    target: Box<dyn Evaluator>,
}

impl Evaluator for ToLower {
    fn expected_type(&self) -> ValueType {
        ValueType::String
    }

    fn eval(&self, file: &FileWrapper) -> Value {
        let target_value = self.target.eval(file);
        match target_value {
            Value::String(s) => s.to_lowercase().into(),
            _ => Value::Empty,
        }
    }
}

struct Trim {
    target: Box<dyn Evaluator>,
}
impl Evaluator for Trim {
    fn expected_type(&self) -> ValueType {
        ValueType::String
    }

    fn eval(&self, file: &FileWrapper) -> Value {
        let target_value = self.target.eval(file);
        match target_value {
            Value::String(s) => s.trim().into(),
            _ => Value::Empty,
        }
    }
}

struct TrimHead {
    target: Box<dyn Evaluator>,
}
impl Evaluator for TrimHead {
    fn expected_type(&self) -> ValueType {
        ValueType::String
    }

    fn eval(&self, file: &FileWrapper) -> Value {
        let target_value = self.target.eval(file);
        match target_value {
            Value::String(s) => s.trim_start().into(),
            _ => Value::Empty,
        }
    }
}
struct TrimTail {
    target: Box<dyn Evaluator>,
}
impl Evaluator for TrimTail {
    fn expected_type(&self) -> ValueType {
        ValueType::String
    }

    fn eval(&self, file: &FileWrapper) -> Value {
        let target_value = self.target.eval(file);
        match target_value {
            Value::String(s) => s.trim_end().into(),
            _ => Value::Empty,
        }
    }
}

struct Reverse {
    target: Box<dyn Evaluator>,
}
impl Evaluator for Reverse {
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

struct Sum {
    target: Box<dyn Evaluator>,
}
impl Evaluator for Sum {
    fn expected_type(&self) -> ValueType {
        ValueType::Number
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::List(value) = self.target.eval(file) else {
            return Value::Empty;
        };
        value
            .items()
            .into_iter()
            .fold(0u64, |acc, item| {
                if let Value::Number(n) = item {
                    acc + n
                } else {
                    acc
                }
            })
            .into()
    }
}

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

impl LambdaFunction {
    fn build(
        &self,
        bindings: &BindingsTypes,
        items_types: &ValueType,
    ) -> Result<Box<dyn Evaluator>, FindItError> {
        let new_bindings = bindings.with(&self.parameter, items_types.clone());

        self.body.build(&new_bindings)
    }
}
impl EvaluatorFactory for MethodInvocation {
    fn build(&self, bindings: &BindingsTypes) -> Result<Box<dyn Evaluator>, FindItError> {
        let target = match &self.target {
            Some(target) => target.build(bindings)?,
            None => Box::new(MeExtractor {}),
        };
        match (&self.method, target.expected_type()) {
            (Method::Length, ValueType::List(_) | ValueType::String | ValueType::Path) => {
                Ok(Box::new(Length { target }))
            }
            (Method::Length, _) => Err(FindItError::BadExpression(
                "Length method can only be applied to List, String or Path types".to_string(),
            )),

            (Method::ToUpper, ValueType::String) => Ok(Box::new(ToUpper { target })),
            (Method::ToUpper, _) => Err(FindItError::BadExpression(
                "ToUpper method can only be applied to String type".to_string(),
            )),
            (Method::ToLower, ValueType::String) => Ok(Box::new(ToLower { target })),
            (Method::ToLower, _) => Err(FindItError::BadExpression(
                "ToLower method can only be applied to String type".to_string(),
            )),
            (Method::Trim, ValueType::String) => Ok(Box::new(Trim { target })),
            (Method::Trim, _) => Err(FindItError::BadExpression(
                "Trim method can only be applied to String type".to_string(),
            )),
            (Method::TrimHead, ValueType::String) => Ok(Box::new(TrimHead { target })),
            (Method::TrimHead, _) => Err(FindItError::BadExpression(
                "TrimHead method can only be applied to String type".to_string(),
            )),
            (Method::TrimTail, ValueType::String) => Ok(Box::new(TrimTail { target })),
            (Method::TrimTail, _) => Err(FindItError::BadExpression(
                "TrimTail method can only be applied to String type".to_string(),
            )),
            (Method::Reverse, ValueType::String) => Ok(Box::new(Reverse { target })),
            (Method::Reverse, _) => Err(FindItError::BadExpression(
                "Reverse method can only be applied to String type".to_string(),
            )),
            (Method::Map(lambda), ValueType::List(input_item_type)) => {
                let input_item_type = input_item_type.clone();
                let lambda_evaluator = lambda.build(bindings, &input_item_type)?;
                let output_item_type = lambda_evaluator.expected_type().clone();
                Ok(Box::new(Map {
                    target,
                    lambda: Rc::new(lambda_evaluator),
                    items_type: Rc::new(output_item_type),
                }))
            }
            (Method::Map(_), _) => Err(FindItError::BadExpression(
                "Map method can only be applied to List type".to_string(),
            )),
            (Method::Filter(lambda), ValueType::List(items_type)) => {
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
            (Method::Filter(_), _) => Err(FindItError::BadExpression(
                "Filter method can only be applied to List type".to_string(),
            )),
            (Method::Sum, ValueType::List(item_type)) => {
                if item_type.deref() != &ValueType::Number {
                    return Err(FindItError::BadExpression(
                        "Sum method can only be applied to List of Number type".to_string(),
                    ));
                }
                Ok(Box::new(Sum { target }))
            }
            (Method::Sum, _) => Err(FindItError::BadExpression(
                "Sum method can only be applied to a List of numbers".to_string(),
            )),
            (Method::Sort, ValueType::List(item_type)) => Ok(Box::new(Sort {
                target,
                item_type: item_type.clone(),
            })),
            (Method::Sort, _) => Err(FindItError::BadExpression(
                "Sort method can only be applied to a List type".to_string(),
            )),
            (Method::SortBy(lambda), ValueType::List(items_type)) => {
                let items_type = items_type.clone();
                let lambda_evaluator = lambda.build(bindings, &items_type)?;

                Ok(Box::new(SortBy {
                    target,
                    lambda: Rc::new(lambda_evaluator),
                    items_type,
                }))
            }
            (Method::SortBy(_), _) => Err(FindItError::BadExpression(
                "Sort by method can only be applied to a List type".to_string(),
            )),
        }
    }
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
    fn test_simple_len() -> Result<(), FindItError> {
        let expr = read_expr(":[1, 2, 3].len()")?;
        let file = &FileWrapper::new(PathBuf::new(), 1);

        assert_eq!(expr.eval(file), Value::Number(3));

        Ok(())
    }

    #[test]
    fn test_simple_sum() -> Result<(), FindItError> {
        let expr = read_expr(":[1, 2, 3].sum()")?;
        let file = &FileWrapper::new(PathBuf::new(), 1);

        assert_eq!(expr.eval(file), Value::Number(6));

        Ok(())
    }

    #[test]
    fn test_simple_sort() -> Result<(), FindItError> {
        let expr = read_expr(":[1, 14, 10].sort()")?;
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
        let expr = read_expr(":[\"abcd\", \"gq\", \"z\", \"12345\"].sortBy({str} {str}.len())")?;
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
    fn test_filter_map_sum() -> Result<(), FindItError> {
        let expr =
            read_expr(":[1, 2, 3, 4, 5, 6].filter({n} {n} % 2 == 0).map({n} {n} * 10).sum()")?;
        let file = &FileWrapper::new(PathBuf::new(), 1);

        assert_eq!(expr.eval(file), Value::Number(120),);

        Ok(())
    }

    #[test]
    fn test_sum_expected_type() -> Result<(), FindItError> {
        let expr = read_expr(":[1, 2, 3, 4, 5, 6].sum()")?;

        assert_eq!(expr.expected_type(), ValueType::Number);

        Ok(())
    }

    #[test]
    fn test_sum_nop_return_empty() -> Result<(), FindItError> {
        let expr = read_expr("files.map({f} {f}.length()).sum()")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn test_sum_ignores_non_numbers() -> Result<(), FindItError> {
        let expr = read_expr("files.map({f} ({f}/ \"first-229.txt\").length()).sum()")?;
        let path = Path::new("tests/test_cases/filter/test_files");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Number(66));

        Ok(())
    }

    #[test]
    fn length_no_string_expr() {
        let err = read_expr("12.LEN()").err();
        assert!(err.is_some())
    }

    #[test]
    fn length_null_str_return_empty() {
        let eval = read_expr("content.len()").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn length_expect_number() {
        let expr = read_expr("\"test\".len()").unwrap();
        assert_eq!(expr.expected_type(), ValueType::Number);
    }

    #[test]
    fn length_return_the_correct_value() {
        let eval = read_expr("\"123\".len()").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Number(3))
    }
    #[test]
    fn test_length_expected_type() -> Result<(), FindItError> {
        test_expected_type("length()", ValueType::Number)
    }

    fn test_expected_type(name: &str, expected: ValueType) -> Result<(), FindItError> {
        let expr = read_expr(name)?;
        let tp = expr.expected_type();

        assert_eq!(tp, expected);

        Ok(())
    }

    #[test]
    fn trim_no_string_expr() {
        let err = read_expr("12.TRIM()").err();
        assert!(err.is_some())
    }

    #[test]
    fn trim_no_args() {
        let err = read_expr("TRIM()").err();
        assert!(err.is_some())
    }

    #[test]
    fn trim_too_many_args() {
        let err = read_expr("\"abc\".TRIM(\"def\")").err();
        assert!(err.is_some())
    }

    #[test]
    fn trim_null_str_return_empty() {
        let eval = read_expr("content.TRIM()").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn trim_expect_string() {
        let expr = read_expr("\"\".TRIM()").unwrap();
        assert_eq!(expr.expected_type(), ValueType::String);
    }

    #[test]
    fn trim_head_null_str_return_empty() {
        let eval = read_expr("content.TRIM_head()").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn trim_head_expect_string() {
        let expr = read_expr("\"\".trim_head()").unwrap();
        assert_eq!(expr.expected_type(), ValueType::String);
    }

    #[test]
    fn trim_tail_null_str_return_empty() {
        let eval = read_expr("content.TRIM_tail()").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn trim_tail_expect_string() {
        let expr = read_expr("\"\".TRIM_tail()").unwrap();
        assert_eq!(expr.expected_type(), ValueType::String);
    }

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
    fn reverse_works() {
        let eval = read_expr("\"123\".REVERSE()").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::String("321".into()))
    }

    #[test]
    fn lower_no_string_expr() {
        let err = read_expr("12.Lower()").err();
        assert!(err.is_some())
    }

    #[test]
    fn lower_no_args() {
        let err = read_expr("lower()").err();
        assert!(err.is_some())
    }

    #[test]
    fn lower_too_many_args() {
        let err = read_expr("\"abc\".lower(\"def\")").err();
        assert!(err.is_some())
    }

    #[test]
    fn lower_null_str_return_empty() {
        let eval = read_expr("content.lower()").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn lower_expect_string() {
        let expr = read_expr("\"test\".lower()").unwrap();
        assert_eq!(expr.expected_type(), ValueType::String);
    }

    #[test]
    fn lower_return_the_correct_value() {
        let eval = read_expr("\"abcDEF\".lower()").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::String("abcdef".into()))
    }

    #[test]
    fn upper_no_string_expr() {
        let err = read_expr("12.Upper()").err();
        assert!(err.is_some())
    }

    #[test]
    fn upper_no_args() {
        let err = read_expr("upper()").err();
        assert!(err.is_some())
    }

    #[test]
    fn upper_too_many_args() {
        let err = read_expr("\"abc\".upper(\"def\")").err();
        assert!(err.is_some())
    }

    #[test]
    fn upper_null_str_return_empty() {
        let eval = read_expr("content.upper()").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);
        assert_eq!(value, Value::Empty)
    }

    #[test]
    fn upper_expect_string() {
        let expr = read_expr("\"test\".upper()").unwrap();
        assert_eq!(expr.expected_type(), ValueType::String);
    }

    #[test]
    fn upper_return_the_correct_value() {
        let eval = read_expr("\"abcDEF\".upper()").unwrap();
        let path = Path::new("no/such/file");
        let wrapper = FileWrapper::new(path.to_path_buf(), 2);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::String("ABCDEF".into()))
    }

    #[test]
    fn test_sort_expected_type() -> Result<(), FindItError> {
        let expr = read_expr(":[1, 2, 3, 4, 5, 6].sort()")?;

        assert_eq!(
            expr.expected_type(),
            ValueType::List(Rc::new(ValueType::Number))
        );

        Ok(())
    }

    #[test]
    fn test_sort_nop_return_empty() -> Result<(), FindItError> {
        let expr = read_expr("files.map({f} {f}.length()).sort()")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn test_sort_by_expected_type() -> Result<(), FindItError> {
        let expr = read_expr(":[1, 2, 3, 4, 5, 6].sort_by({f} {f})")?;

        assert_eq!(
            expr.expected_type(),
            ValueType::List(Rc::new(ValueType::Number))
        );

        Ok(())
    }

    #[test]
    fn test_sort_by_nop_return_empty() -> Result<(), FindItError> {
        let expr = read_expr("files.sort_by({f} {f}.length())")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn length_no_string_trim_head() {
        let err = read_expr("12.trimHead()").err();
        assert!(err.is_some())
    }

    #[test]
    fn length_no_string_trim_tail() {
        let err = read_expr("12.trimTail()").err();
        assert!(err.is_some())
    }

    #[test]
    fn length_no_string_reverse() {
        let err = read_expr("12.reverse()").err();
        assert!(err.is_some())
    }

    #[test]
    fn length_no_list_map() {
        let err = read_expr("12.map({f} {f})").err();
        assert!(err.is_some())
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

    #[test]
    fn length_no_list_sum() {
        let err = read_expr("12.sum()").err();
        assert!(err.is_some())
    }

    #[test]
    fn length_no_number_sum() {
        let err = read_expr(":[\"a\", \"b\"].sum()").err();
        assert!(err.is_some())
    }

    #[test]
    fn length_no_list_sort() {
        let err = read_expr("12.sort()").err();
        assert!(err.is_some())
    }

    #[test]
    fn length_no_list_sort_by() {
        let err = read_expr("12.sort_by({f} {f})").err();
        assert!(err.is_some())
    }
}
