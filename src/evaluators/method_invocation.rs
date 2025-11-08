use std::{
    fs::File,
    io::{BufRead, BufReader},
    ops::Deref,
    rc::Rc,
};

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

struct SkipString {
    target: Box<dyn Evaluator>,
    by: Box<dyn Evaluator>,
}
impl Evaluator for SkipString {
    fn expected_type(&self) -> ValueType {
        ValueType::String
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::String(target_value) = self.target.eval(file) else {
            return Value::Empty;
        };
        let Value::Number(by_value) = self.by.eval(file) else {
            return Value::Empty;
        };
        target_value
            .chars()
            .skip(by_value as usize)
            .collect::<String>()
            .into()
    }
}

struct SkipList {
    target: Box<dyn Evaluator>,
    by: Box<dyn Evaluator>,
    items_type: Rc<ValueType>,
}
impl Evaluator for SkipList {
    fn expected_type(&self) -> ValueType {
        ValueType::List(self.items_type.clone())
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::List(target_value) = self.target.eval(file) else {
            return Value::Empty;
        };
        let Value::Number(by_value) = self.by.eval(file) else {
            return Value::Empty;
        };
        let iter = target_value.items().into_iter().skip(by_value as usize);
        Value::List(List::new_lazy(self.items_type.clone(), iter))
    }
}

struct TakeString {
    target: Box<dyn Evaluator>,
    limit: Box<dyn Evaluator>,
}
impl Evaluator for TakeString {
    fn expected_type(&self) -> ValueType {
        ValueType::String
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::String(target_value) = self.target.eval(file) else {
            return Value::Empty;
        };
        let Value::Number(limit_value) = self.limit.eval(file) else {
            return Value::Empty;
        };
        target_value
            .chars()
            .take(limit_value as usize)
            .collect::<String>()
            .into()
    }
}

struct TakeList {
    target: Box<dyn Evaluator>,
    limit: Box<dyn Evaluator>,
    items_type: Rc<ValueType>,
}
impl Evaluator for TakeList {
    fn expected_type(&self) -> ValueType {
        ValueType::List(self.items_type.clone())
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::List(target_value) = self.target.eval(file) else {
            return Value::Empty;
        };
        let Value::Number(limit) = self.limit.eval(file) else {
            return Value::Empty;
        };
        let iter = target_value.items().into_iter().take(limit as usize);
        Value::List(List::new_lazy(self.items_type.clone(), iter))
    }
}

struct Join {
    target: Box<dyn Evaluator>,
    delimiter: Option<Box<dyn Evaluator>>,
}
impl Evaluator for Join {
    fn expected_type(&self) -> ValueType {
        ValueType::String
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::List(target_value) = self.target.eval(file) else {
            return Value::Empty;
        };
        let delimiter = if let Some(delim_eval) = &self.delimiter {
            let Value::String(delim_value) = delim_eval.eval(file) else {
                return Value::Empty;
            };
            delim_value
        } else {
            ",".to_string()
        };
        target_value
            .items()
            .into_iter()
            .map(|f| f.to_string())
            .join(&delimiter)
            .into()
    }
}

struct Split {
    target: Box<dyn Evaluator>,
    delimiter: Box<dyn Evaluator>,
}
impl Evaluator for Split {
    fn expected_type(&self) -> ValueType {
        ValueType::List(Rc::new(ValueType::String))
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::String(target_value) = self.target.eval(file) else {
            return Value::Empty;
        };
        let Value::String(delimiter) = self.delimiter.eval(file) else {
            return Value::Empty;
        };
        if delimiter.is_empty() {
            return Value::Empty;
        }
        let items = target_value
            .split(&delimiter)
            .map(|s| Value::String(s.to_string()));
        Value::List(List::new_eager(Rc::new(ValueType::String), items))
    }
}
struct LinesString {
    target: Box<dyn Evaluator>,
}
impl Evaluator for LinesString {
    fn expected_type(&self) -> ValueType {
        ValueType::List(Rc::new(ValueType::String))
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::String(str) = self.target.eval(file) else {
            return Value::Empty;
        };
        let items = str.lines().map(|s| Value::String(s.to_string()));
        Value::List(List::new_eager(Rc::new(ValueType::String), items))
    }
}

struct LinesFile {
    target: Box<dyn Evaluator>,
}
impl Evaluator for LinesFile {
    fn expected_type(&self) -> ValueType {
        ValueType::List(Rc::new(ValueType::String))
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::Path(path) = self.target.eval(file) else {
            return Value::Empty;
        };
        let Ok(file) = File::open(path) else {
            return Value::Empty;
        };
        let buf = BufReader::new(file);
        let items = buf.lines().map_while(Result::ok).map(Value::String);
        Value::List(List::new_lazy(Rc::new(ValueType::String), items))
    }
}
struct StringWords {
    target: Box<dyn Evaluator>,
}
impl Evaluator for StringWords {
    fn expected_type(&self) -> ValueType {
        ValueType::List(Rc::new(ValueType::String))
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::String(target_value) = self.target.eval(file) else {
            return Value::Empty;
        };
        let items = target_value
            .split_whitespace()
            .map(|s| Value::String(s.to_string()));
        Value::List(List::new_eager(Rc::new(ValueType::String), items))
    }
}
struct FileWords {
    target: Box<dyn Evaluator>,
}
impl Evaluator for FileWords {
    fn expected_type(&self) -> ValueType {
        ValueType::List(Rc::new(ValueType::String))
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let Value::Path(path) = self.target.eval(file) else {
            return Value::Empty;
        };
        let Ok(file) = File::open(path) else {
            return Value::Empty;
        };
        let buf = BufReader::new(file);
        let items = buf.lines().map_while(Result::ok).flat_map(|s| {
            s.split_whitespace()
                .map(|s| Value::String(s.into()))
                .collect::<Vec<_>>()
        });

        Value::List(List::new_lazy(Rc::new(ValueType::String), items))
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
            (Method::Skip(by), ValueType::String) => {
                let by = by.build(bindings)?;
                if by.expected_type() != ValueType::Number {
                    return Err(FindItError::BadExpression(
                        "Skip method argument must be a Number".to_string(),
                    ));
                }
                Ok(Box::new(SkipString { target, by }))
            }
            (Method::Skip(by), ValueType::List(items_type)) => {
                let by = by.build(bindings)?;
                if by.expected_type() != ValueType::Number {
                    return Err(FindItError::BadExpression(
                        "Skip method argument must be a Number".to_string(),
                    ));
                }
                Ok(Box::new(SkipList {
                    target,
                    by,
                    items_type: items_type.clone(),
                }))
            }
            (Method::Skip(_), _) => Err(FindItError::BadExpression(
                "Skip method can only be applied to String or List types".to_string(),
            )),
            (Method::Take(limit), ValueType::String) => {
                let limit = limit.build(bindings)?;
                if limit.expected_type() != ValueType::Number {
                    return Err(FindItError::BadExpression(
                        "Take method argument must be a Number".to_string(),
                    ));
                }
                Ok(Box::new(TakeString { target, limit }))
            }
            (Method::Take(limit), ValueType::List(items_type)) => {
                let limit = limit.build(bindings)?;
                if limit.expected_type() != ValueType::Number {
                    return Err(FindItError::BadExpression(
                        "Take method argument must be a Number".to_string(),
                    ));
                }
                Ok(Box::new(TakeList {
                    target,
                    limit,
                    items_type: items_type.clone(),
                }))
            }
            (Method::Take(_), _) => Err(FindItError::BadExpression(
                "Take method can only be applied to String or List types".to_string(),
            )),
            (Method::Join(delimiter), ValueType::List(_)) => {
                let delimiter = match delimiter {
                    Some(delim) => {
                        let delim = delim.build(bindings)?;
                        if delim.expected_type() != ValueType::String {
                            return Err(FindItError::BadExpression(
                                "Join method delimiter must be a String".to_string(),
                            ));
                        }
                        Some(delim)
                    }
                    None => None,
                };

                Ok(Box::new(Join { target, delimiter }))
            }
            (Method::Join(_), _) => Err(FindItError::BadExpression(
                "Join method can only be applied to List type".to_string(),
            )),
            (Method::Split(delimiter), ValueType::String) => {
                let delimiter = delimiter.build(bindings)?;
                if delimiter.expected_type() != ValueType::String {
                    return Err(FindItError::BadExpression(
                        "Split method delimiter must be a String".to_string(),
                    ));
                }
                Ok(Box::new(Split { target, delimiter }))
            }
            (Method::Split(_), _) => Err(FindItError::BadExpression(
                "Split method can only be applied to String type".to_string(),
            )),
            (Method::Lines, ValueType::String) => Ok(Box::new(LinesString { target })),
            (Method::Lines, ValueType::Path) => Ok(Box::new(LinesFile { target })),
            (Method::Lines, _) => Err(FindItError::BadExpression(
                "Lines method can only be applied to String or Path types".to_string(),
            )),
            (Method::Words, ValueType::String) => Ok(Box::new(StringWords { target })),
            (Method::Words, ValueType::Path) => Ok(Box::new(FileWords { target })),
            (Method::Words, _) => Err(FindItError::BadExpression(
                "Words method can only be applied to String or Path types".to_string(),
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

    #[test]
    fn test_simple_skip() -> Result<(), FindItError> {
        let expr = read_expr("\"abc\".skip(2)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::String("c".into()));

        Ok(())
    }

    #[test]
    fn test_skip_large_number() -> Result<(), FindItError> {
        let expr = read_expr("\"abc\".skip(100)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::String("".into()));

        Ok(())
    }


    #[test]
    fn test_skip_empty_string() -> Result<(), FindItError> {
        let expr = read_expr("content.skip(100)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }


    #[test]
    fn test_skip_empty_number() -> Result<(), FindItError> {
        let expr = read_expr("\"abc\".skip(size)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn skip_return_type() -> Result<(), FindItError> {
        let expr = read_expr("\"abc\".skip(2)")?;

        assert_eq!(expr.expected_type(), ValueType::String);

        Ok(())
    }

    #[test]
    fn length_no_string_skip() {
        let err = read_expr("12.skip(2)").err();
        assert!(err.is_some())
    }

    #[test]
    fn length_no_number_skip() {
        let err = read_expr("\"abc\".skip(\"a\")").err();
        assert!(err.is_some())
    }

    #[test]
    fn test_simple_take() -> Result<(), FindItError> {
        let expr = read_expr("\"abc\".take(2)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::String("ab".into()));

        Ok(())
    }

    #[test]
    fn test_take_large_number() -> Result<(), FindItError> {
        let expr = read_expr("\"abc\".take(100)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::String("abc".into()));

        Ok(())
    }

    #[test]
    fn take_return_type() -> Result<(), FindItError> {
        let expr = read_expr("\"abc\".take(2)")?;

        assert_eq!(expr.expected_type(), ValueType::String);

        Ok(())
    }

    #[test]
    fn length_no_string_take() {
        let err = read_expr("12.take(2)").err();
        assert!(err.is_some())
    }

    #[test]
    fn length_no_number_take() {
        let err = read_expr("\"abc\".take(\"a\")").err();
        assert!(err.is_some())
    }


    #[test]
    fn test_take_empty_string() -> Result<(), FindItError> {
        let expr = read_expr("content.take(100)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }


    #[test]
    fn test_take_empty_number() -> Result<(), FindItError> {
        let expr = read_expr("\"abc\".take(size)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }


    #[test]
    fn test_skip_list() -> Result<(), FindItError> {
        let expr = read_expr(":[1, 2, 4, 5].skip(2)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(
            expr.eval(file),
            Value::List(List::new_eager(
                Rc::new(ValueType::Number),
                vec![Value::Number(4), Value::Number(5),].into_iter(),
            ))
        );

        Ok(())
    }

    #[test]
    fn test_skip_list_large_number() -> Result<(), FindItError> {
        let expr = read_expr(":[1, 2, 4, 5].skip(100)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(
            expr.eval(file),
            Value::List(List::new_eager(
                Rc::new(ValueType::Number),
                vec![].into_iter(),
            ))
        );

        Ok(())
    }

    #[test]
    fn skip_list_return_type() -> Result<(), FindItError> {
        let expr = read_expr(":[1, 2, 4, 5].skip(2)")?;

        assert_eq!(
            expr.expected_type(),
            ValueType::List(Rc::new(ValueType::Number))
        );

        Ok(())
    }

    #[test]
    fn skip_list_nan() {
        let err = read_expr(":[1, 2, 4, 5].skip(\"a\")").err();
        assert!(err.is_some())
    }

    #[test]
    fn test_skip_list_empty_string() -> Result<(), FindItError> {
        let expr = read_expr("files.skip(100)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }


    #[test]
    fn test_skip_list_empty_number() -> Result<(), FindItError> {
        let expr = read_expr(":[1, 3].skip(size)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn test_simple_take_list() -> Result<(), FindItError> {
        let expr = read_expr(":[1, 2, 3].take(2)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(
            expr.eval(file),
            Value::List(List::new_eager(
                Rc::new(ValueType::Number),
                vec![Value::Number(1), Value::Number(2),].into_iter(),
            ))
        );

        Ok(())
    }

    #[test]
    fn test_take_list_large_number() -> Result<(), FindItError> {
        let expr = read_expr(":[1, 2, 3].take(100)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(
            expr.eval(file),
            Value::List(List::new_eager(
                Rc::new(ValueType::Number),
                vec![Value::Number(1), Value::Number(2), Value::Number(3),].into_iter(),
            ))
        );

        Ok(())
    }

    #[test]
    fn take_list_return_type() -> Result<(), FindItError> {
        let expr = read_expr(":[1, 2, 3].take(2)")?;

        assert_eq!(
            expr.expected_type(),
            ValueType::List(Rc::new(ValueType::Number))
        );

        Ok(())
    }

    #[test]
    fn take_list_nan_error() {
        let err = read_expr(":[1, 2, 3].take(\"a\")").err();
        assert!(err.is_some())
    }


    #[test]
    fn test_take_list_empty_string() -> Result<(), FindItError> {
        let expr = read_expr("files.take(100)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }


    #[test]
    fn test_take_list_empty_number() -> Result<(), FindItError> {
        let expr = read_expr(":[1, 3].take(size)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }


    #[test]
    fn test_join_no_arg() -> Result<(), FindItError> {
        let expr = read_expr(":[1, 2, 4, 5].join()")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::String("1,2,4,5".into()));

        Ok(())
    }

    #[test]
    fn test_join_with_arg() -> Result<(), FindItError> {
        let expr = read_expr(":[1, 2, 4, 5].join(\";\")")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::String("1;2;4;5".into()));

        Ok(())
    }

    #[test]
    fn join_return_type() -> Result<(), FindItError> {
        let expr = read_expr(":[1, 2, 4, 5].join(\";\")")?;

        assert_eq!(expr.expected_type(), ValueType::String);

        Ok(())
    }


    #[test]
    fn test_join_no_target() -> Result<(), FindItError> {
        let expr = read_expr("files.join()")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn test_join_with_empty_arg() -> Result<(), FindItError> {
        let expr = read_expr(":[1, 2, 4, 5].join(content)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn join_no_list() {
        let err = read_expr("123.join(\"a\")").err();
        assert!(err.is_some())
    }

    #[test]
    fn empty_join_no_list() {
        let err = read_expr("123.join()").err();
        assert!(err.is_some())
    }

    #[test]
    fn join_no_string() {
        let err = read_expr(":[1, 2, 3].join(123)").err();
        assert!(err.is_some())
    }

    #[test]
    fn test_split() -> Result<(), FindItError> {
        let expr = read_expr("\"a|b|c\".split(\"|\")")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(
            expr.eval(file),
            Value::List(List::new_eager(
                Rc::new(ValueType::String),
                vec![
                    Value::String("a".into()),
                    Value::String("b".into()),
                    Value::String("c".into())
                ]
                .into_iter(),
            ))
        );

        Ok(())
    }

    #[test]
    fn test_split_no_delim() -> Result<(), FindItError> {
        let expr = read_expr("\"a|b|c\".split(\"\")")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(expr.eval(file), Value::Empty);

        Ok(())
    }

    #[test]
    fn test_split_no_target() -> Result<(), FindItError> {
        let expr = read_expr("content.split(\"|\")")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(
            expr.eval(file),
            Value::Empty
        );

        Ok(())
    }


    #[test]
    fn test_split_empty_delim() -> Result<(), FindItError> {
        let expr = read_expr("\"a|b|c\".split(content)")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(
            expr.eval(file),
            Value::Empty
        );

        Ok(())
    }

    #[test]
    fn test_split_return_type() -> Result<(), FindItError> {
        let expr = read_expr("\"a|b|c\".split(\"\")")?;

        assert_eq!(
            expr.expected_type(),
            ValueType::List(Rc::new(ValueType::String))
        );

        Ok(())
    }

    #[test]
    fn test_split_no_str() {
        let expr = read_expr("\"a|b|c\".split(12)").err();

        assert!(expr.is_some());
    }

    #[test]
    fn test_split_no_str_two() {
        let expr = read_expr("12.split(\"a\")").err();

        assert!(expr.is_some());
    }

    #[test]
    fn test_lines_string() -> Result<(), FindItError> {
        let expr = read_expr("\"one\ntwo\nthree\n\".lines()")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(
            expr.eval(file),
            Value::List(List::new_eager(
                Rc::new(ValueType::String),
                vec![
                    Value::String("one".into()),
                    Value::String("two".into()),
                    Value::String("three".into())
                ]
                .into_iter(),
            ))
        );

        Ok(())
    }

    #[test]
    fn test_lines_string_no_target() -> Result<(), FindItError> {
        let expr = read_expr("content.lines()")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(
            expr.eval(file),
            Value::Empty
        );

        Ok(())
    }

    #[test]
    fn test_lines_string_return_type() -> Result<(), FindItError> {
        let expr = read_expr("\"abc\".lines()")?;

        assert_eq!(
            expr.expected_type(),
            ValueType::List(Rc::new(ValueType::String))
        );

        Ok(())
    }
    #[test]
    fn test_lines_number() {
        let expr = read_expr("12.lines()").err();

        assert!(expr.is_some());
    }

    #[test]
    fn test_lines_file() -> Result<(), FindItError> {
        let expr = read_expr("lines()")?;
        let path = Path::new("tests/test_cases/display/test_files/week-362.txt");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(
            expr.eval(file),
            Value::List(List::new_eager(
                Rc::new(ValueType::String),
                vec![
                    Value::String("quo eligendi amet harum ullam minus quasi ut.".into()),
                    Value::String("magni neque sed est incidunt expedita.".into()),
                    Value::String(
                        "quia quasi illo perferendis doloremque illum qui voluptas ullam.".into()
                    ),
                    Value::String("ab nulla nobis maiores nobis beatae velit ea quia.".into()),
                    Value::String("adipisci debitis facilis molestiae soluta repellat aut.".into()),
                    Value::String("vero libero repudiandae fugiat ducimus occaecati.".into()),
                    Value::String("".into()),
                ]
                .into_iter(),
            ))
        );

        Ok(())
    }

    #[test]
    fn test_lines_file_return_type() -> Result<(), FindItError> {
        let expr = read_expr("lines()")?;

        assert_eq!(
            expr.expected_type(),
            ValueType::List(Rc::new(ValueType::String))
        );

        Ok(())
    }

    #[test]
    fn test_words_string() -> Result<(), FindItError> {
        let expr = read_expr("\"  one\ntwo  three \n    \".words()")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(
            expr.eval(file),
            Value::List(List::new_eager(
                Rc::new(ValueType::String),
                vec![
                    Value::String("one".into()),
                    Value::String("two".into()),
                    Value::String("three".into())
                ]
                .into_iter(),
            ))
        );

        Ok(())
    }

    #[test]
    fn test_words_string_no_target() -> Result<(), FindItError> {
        let expr = read_expr("content.words()")?;
        let path = Path::new("no/such/file");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(
            expr.eval(file),
            Value::Empty
        );

        Ok(())
    }

    #[test]
    fn test_words_string_return_type() -> Result<(), FindItError> {
        let expr = read_expr("\"abc\".words()")?;

        assert_eq!(
            expr.expected_type(),
            ValueType::List(Rc::new(ValueType::String))
        );

        Ok(())
    }

    #[test]
    fn test_words_number() {
        let expr = read_expr("12.words()").err();

        assert!(expr.is_some());
    }

    #[test]
    fn test_words_file() -> Result<(), FindItError> {
        let expr = read_expr("words()")?;
        let path = Path::new("tests/test_cases/display/test_files/week-362.txt");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(
            expr.eval(file),
            Value::List(List::new_eager(
                Rc::new(ValueType::String),
                vec![
                    Value::String("quo".into()),
                    Value::String("eligendi".into()),
                    Value::String("amet".into()),
                    Value::String("harum".into()),
                    Value::String("ullam".into()),
                    Value::String("minus".into()),
                    Value::String("quasi".into()),
                    Value::String("ut.".into()),
                    Value::String("magni".into()),
                    Value::String("neque".into()),
                    Value::String("sed".into()),
                    Value::String("est".into()),
                    Value::String("incidunt".into()),
                    Value::String("expedita.".into()),
                    Value::String("quia".into()),
                    Value::String("quasi".into()),
                    Value::String("illo".into()),
                    Value::String("perferendis".into()),
                    Value::String("doloremque".into()),
                    Value::String("illum".into()),
                    Value::String("qui".into()),
                    Value::String("voluptas".into()),
                    Value::String("ullam.".into()),
                    Value::String("ab".into()),
                    Value::String("nulla".into()),
                    Value::String("nobis".into()),
                    Value::String("maiores".into()),
                    Value::String("nobis".into()),
                    Value::String("beatae".into()),
                    Value::String("velit".into()),
                    Value::String("ea".into()),
                    Value::String("quia.".into()),
                    Value::String("adipisci".into()),
                    Value::String("debitis".into()),
                    Value::String("facilis".into()),
                    Value::String("molestiae".into()),
                    Value::String("soluta".into()),
                    Value::String("repellat".into()),
                    Value::String("aut.".into()),
                    Value::String("vero".into()),
                    Value::String("libero".into()),
                    Value::String("repudiandae".into()),
                    Value::String("fugiat".into()),
                    Value::String("ducimus".into()),
                    Value::String("occaecati.".into()),
                ]
                .into_iter(),
            ))
        );

        Ok(())
    }

    #[test]
    fn test_words_file_no_target() -> Result<(), FindItError> {
        let expr = read_expr("parent.words()")?;
        let path = Path::new("/");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(
            expr.eval(file),
            Value::Empty,
        );

        Ok(())
    }


    #[test]
    fn test_words_file_target_not_a_file() -> Result<(), FindItError> {
        let expr = read_expr("parent.words()")?;
        let path = Path::new(".");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(
            expr.eval(file),
            Value::Empty,
        );

        Ok(())
    }

    #[test]
    fn test_words_file_return_type() -> Result<(), FindItError> {
        let expr = read_expr("words()")?;

        assert_eq!(
            expr.expected_type(),
            ValueType::List(Rc::new(ValueType::String))
        );

        Ok(())
    }


    #[test]
    fn test_lines_file_no_target() -> Result<(), FindItError> {
        let expr = read_expr("parent.lines()")?;
        let path = Path::new("/");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(
            expr.eval(file),
            Value::Empty,
        );

        Ok(())
    }


    #[test]
    fn test_lines_file_target_not_a_file() -> Result<(), FindItError> {
        let expr = read_expr("parent.lines()")?;
        let path = Path::new(".");
        let file = &FileWrapper::new(path.to_path_buf(), 1);

        assert_eq!(
            expr.eval(file),
            Value::Empty,
        );

        Ok(())
    }

}
