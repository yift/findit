use std::path::Path;

use chrono::DateTime;

use crate::{
    errors::FindItError,
    evaluators::expr::{BindingsTypes, Evaluator, EvaluatorFactory},
    file_wrapper::FileWrapper,
    parser::{
        ast::{
            as_cast::{As, CastType},
            expression::Expression,
        },
        parse_expression,
    },
    value::{Value, ValueType},
};

struct CastToBool {
    expr: Box<dyn Evaluator>,
}
impl Evaluator for CastToBool {
    fn expected_type(&self) -> ValueType {
        ValueType::Bool
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        match self.expr.eval(file) {
            Value::String(str) => {
                matches!(str.to_lowercase().as_str(), "yes" | "true" | "y" | "t").into()
            }
            Value::Bool(b) => b.into(),
            Value::Date(_) => true.into(),
            Value::Empty => Value::Empty,
            Value::Number(n) => (n != 0).into(),
            Value::Path(p) => p.exists().into(),
            Value::List(l) => l.has_items().into(),
        }
    }
}

struct CastToString {
    expr: Box<dyn Evaluator>,
}
impl Evaluator for CastToString {
    fn expected_type(&self) -> ValueType {
        ValueType::String
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        self.expr.eval(file).to_string().into()
    }
}

struct CastToNumber {
    expr: Box<dyn Evaluator>,
}
impl Evaluator for CastToNumber {
    fn expected_type(&self) -> ValueType {
        ValueType::Number
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        match self.expr.eval(file) {
            Value::Bool(true) => Value::Number(1),
            Value::Bool(false) => Value::Number(0),
            Value::Empty => Value::Number(0),
            Value::Date(dt) => match dt.timestamp().try_into() {
                Ok(num) => Value::Number(num),
                Err(_) => Value::Empty,
            },
            Value::String(str) => match str.parse::<u64>() {
                Ok(num) => Value::Number(num),
                Err(_) => Value::Empty,
            },
            Value::Number(n) => Value::Number(n),
            Value::List(l) => l.count().into(),
            Value::Path(_) => Value::Empty,
        }
    }
}

struct CastToDate {
    expr: Box<dyn Evaluator>,
}
impl Evaluator for CastToDate {
    fn expected_type(&self) -> ValueType {
        ValueType::Date
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        match self.expr.eval(file) {
            Value::Bool(_) | Value::Empty => Value::Empty,
            Value::Date(dt) => Value::Date(dt),
            Value::String(str) => match parse_expression(&format!("@({})", str)) {
                Ok(Expression::Literal(Value::Date(dt))) => Value::Date(dt),
                _ => Value::Empty,
            },
            Value::Number(n) => match n.try_into() {
                Ok(secs) => match DateTime::from_timestamp(secs, 0) {
                    Some(dt) => Value::Date(dt.into()),
                    None => Value::Empty,
                },
                Err(_) => Value::Empty,
            },
            Value::Path(p) => match p.metadata() {
                Ok(m) => match m.accessed() {
                    Ok(tm) => Value::Date(tm.into()),
                    Err(_) => Value::Empty,
                },
                Err(_) => Value::Empty,
            },
            Value::List(_) => Value::Empty,
        }
    }
}

struct CastToPath {
    expr: Box<dyn Evaluator>,
}
impl Evaluator for CastToPath {
    fn expected_type(&self) -> ValueType {
        ValueType::Path
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        match self.expr.eval(file) {
            Value::Bool(_) | Value::Empty | Value::Date(_) | Value::Number(_) | Value::List(_) => {
                Value::Empty
            }
            Value::Path(p) => Value::Path(p),
            Value::String(s) => Value::Path(Path::new(&s).to_path_buf()),
        }
    }
}

impl EvaluatorFactory for As {
    fn build(&self, bindings: &BindingsTypes) -> Result<Box<dyn Evaluator>, FindItError> {
        let expr = self.expression.build(bindings)?;
        match self.cast_type {
            CastType::Bool => Ok(Box::new(CastToBool { expr })),
            CastType::String => Ok(Box::new(CastToString { expr })),
            CastType::Number => Ok(Box::new(CastToNumber { expr })),
            CastType::Date => Ok(Box::new(CastToDate { expr })),
            CastType::Path => Ok(Box::new(CastToPath { expr })),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::evaluators::expr::read_expr;

    use super::*;

    #[test]
    fn test_identity_cast_to_bool() -> Result<(), FindItError> {
        let sql = "true as bool";
        let eval = read_expr(sql)?;
        let file = Path::new("/").to_path_buf();
        let wrapper = FileWrapper::new(file, 1);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Bool(true));
        Ok(())
    }

    #[test]
    fn test_string_cast_to_bool_true() -> Result<(), FindItError> {
        let sql = "\"yes\" as bool";
        let eval = read_expr(sql)?;
        let file = Path::new("/").to_path_buf();
        let wrapper = FileWrapper::new(file, 1);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Bool(true));
        Ok(())
    }

    #[test]
    fn test_string_cast_to_bool_false() -> Result<(), FindItError> {
        let sql = "\"no\" as bool";
        let eval = read_expr(sql)?;
        let file = Path::new("/").to_path_buf();
        let wrapper = FileWrapper::new(file, 1);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Bool(false));
        Ok(())
    }

    #[test]
    fn test_date_cast_to_bool() -> Result<(), FindItError> {
        let sql = "now() as bool";
        let eval = read_expr(sql)?;
        let file = Path::new("/").to_path_buf();
        let wrapper = FileWrapper::new(file, 1);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Bool(true));
        Ok(())
    }

    #[test]
    fn test_empty_cast_to_bool() -> Result<(), FindItError> {
        let sql = "content as bool";
        let eval = read_expr(sql)?;
        let file = Path::new("/no/such/file").to_path_buf();
        let wrapper = FileWrapper::new(file, 1);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Empty);
        Ok(())
    }

    #[test]
    fn test_number_cast_to_bool_true() -> Result<(), FindItError> {
        let sql = "10 as bool";
        let eval = read_expr(sql)?;
        let file = Path::new("/no/such/file").to_path_buf();
        let wrapper = FileWrapper::new(file, 1);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Bool(true));
        Ok(())
    }

    #[test]
    fn test_number_cast_to_bool_false() -> Result<(), FindItError> {
        let sql = "0 as bool";
        let eval = read_expr(sql)?;
        let file = Path::new("/no/such/file").to_path_buf();
        let wrapper = FileWrapper::new(file, 1);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Bool(false));
        Ok(())
    }

    #[test]
    fn path_number_cast_to_bool_false() -> Result<(), FindItError> {
        let sql = "self as bool";
        let eval = read_expr(sql)?;
        let file = Path::new("/no/such/file").to_path_buf();
        let wrapper = FileWrapper::new(file, 1);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Bool(false));
        Ok(())
    }

    #[test]
    fn path_number_cast_to_bool_true() -> Result<(), FindItError> {
        let sql = "self as bool";
        let eval = read_expr(sql)?;
        let file = Path::new(".").to_path_buf();
        let wrapper = FileWrapper::new(file, 1);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Bool(true));
        Ok(())
    }

    #[test]
    fn empty_list_cast_to_bool_false() -> Result<(), FindItError> {
        let sql = "[] as bool";
        let eval = read_expr(sql)?;
        let file = Path::new("/no/such/file").to_path_buf();
        let wrapper = FileWrapper::new(file, 1);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Bool(false));
        Ok(())
    }

    #[test]
    fn non_empty_list_cast_to_bool_true() -> Result<(), FindItError> {
        let sql = "[1] as bool";
        let eval = read_expr(sql)?;
        let file = Path::new("/no/such/file").to_path_buf();
        let wrapper = FileWrapper::new(file, 1);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Bool(true));
        Ok(())
    }

    #[test]
    fn bool_cast_to_string() -> Result<(), FindItError> {
        let sql = "true as text";
        let eval = read_expr(sql)?;
        let file = Path::new(".").to_path_buf();
        let wrapper = FileWrapper::new(file, 1);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::String("true".into()));
        Ok(())
    }

    #[test]
    fn bool_cast_to_number() -> Result<(), FindItError> {
        let sql = "true as integer";
        let eval = read_expr(sql)?;
        let file = Path::new(".").to_path_buf();
        let wrapper = FileWrapper::new(file, 1);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Number(1));
        Ok(())
    }

    #[test]
    fn bool_false_cast_to_number() -> Result<(), FindItError> {
        let sql = "false as integer";
        let eval = read_expr(sql)?;
        let file = Path::new(".").to_path_buf();
        let wrapper = FileWrapper::new(file, 1);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Number(0));
        Ok(())
    }

    #[test]
    fn empty_cast_to_number() -> Result<(), FindItError> {
        let sql = "content as integer";
        let eval = read_expr(sql)?;
        let file = Path::new("/no/such/file").to_path_buf();
        let wrapper = FileWrapper::new(file, 1);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Number(0));
        Ok(())
    }

    #[test]
    fn string_cast_to_number() -> Result<(), FindItError> {
        let sql = "\"100\" as number";
        let eval = read_expr(sql)?;
        let file = Path::new("/no/such/file").to_path_buf();
        let wrapper = FileWrapper::new(file, 1);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Number(100));
        Ok(())
    }

    #[test]
    fn string_cast_to_number_fails() -> Result<(), FindItError> {
        let sql = "\"hello\" as number";
        let eval = read_expr(sql)?;
        let file = Path::new("/no/such/file").to_path_buf();
        let wrapper = FileWrapper::new(file, 1);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Empty);
        Ok(())
    }

    #[test]
    fn date_cast_to_number() -> Result<(), FindItError> {
        let sql = "@(1970-01-02) as number";
        let eval = read_expr(sql)?;
        let file = Path::new("/no/such/file").to_path_buf();
        let wrapper = FileWrapper::new(file, 1);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Number(82800));
        Ok(())
    }

    #[test]
    fn date_cast_to_number_fails() -> Result<(), FindItError> {
        let sql = "@(1960-01-02) as number";
        let eval = read_expr(sql)?;
        let file = Path::new("/no/such/file").to_path_buf();
        let wrapper = FileWrapper::new(file, 1);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Empty);
        Ok(())
    }

    #[test]
    fn number_cast_to_number() -> Result<(), FindItError> {
        let sql = "30041 as number";
        let eval = read_expr(sql)?;
        let file = Path::new("/no/such/file").to_path_buf();
        let wrapper = FileWrapper::new(file, 1);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Number(30041));
        Ok(())
    }

    #[test]
    fn path_cast_to_number() -> Result<(), FindItError> {
        let sql = "parent as number";
        let eval = read_expr(sql)?;
        let file = Path::new("/no/such/file").to_path_buf();
        let wrapper = FileWrapper::new(file, 1);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Empty);
        Ok(())
    }

    #[test]
    fn list_cast_to_number() -> Result<(), FindItError> {
        let sql = "[1, 2, 12] as number";
        let eval = read_expr(sql)?;
        let file = Path::new("/no/such/file").to_path_buf();
        let wrapper = FileWrapper::new(file, 1);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Number(3));
        Ok(())
    }

    #[test]
    fn bool_cast_to_date() -> Result<(), FindItError> {
        let sql = "true as time";
        let eval = read_expr(sql)?;
        let file = Path::new("/no/such/file").to_path_buf();
        let wrapper = FileWrapper::new(file, 1);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Empty);
        Ok(())
    }

    #[test]
    fn empty_cast_to_date() -> Result<(), FindItError> {
        let sql = "content as time";
        let eval = read_expr(sql)?;
        let file = Path::new("/no/such/file").to_path_buf();
        let wrapper = FileWrapper::new(file, 1);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Empty);
        Ok(())
    }

    #[test]
    fn date_cast_to_date() -> Result<(), FindItError> {
        let sql = "@(1970-01-02) as time";
        let eval = read_expr(sql)?;
        let file = Path::new("/no/such/file").to_path_buf();
        let wrapper = FileWrapper::new(file, 1);
        let value = eval.eval(&wrapper);

        assert_eq!(
            value,
            Value::Date(DateTime::from_timestamp(82800, 0).unwrap().into())
        );
        Ok(())
    }

    #[test]
    fn string_cast_to_date_success() -> Result<(), FindItError> {
        let sql = "\"1970-01-02\" as time";
        let eval = read_expr(sql)?;
        let file = Path::new("/no/such/file").to_path_buf();
        let wrapper = FileWrapper::new(file, 1);
        let value = eval.eval(&wrapper);

        assert_eq!(
            value,
            Value::Date(DateTime::from_timestamp(82800, 0).unwrap().into())
        );
        Ok(())
    }

    #[test]
    fn string_cast_to_date_fails() -> Result<(), FindItError> {
        let sql = "\"hello\" as time";
        let eval = read_expr(sql)?;
        let file = Path::new("/no/such/file").to_path_buf();
        let wrapper = FileWrapper::new(file, 1);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Empty);
        Ok(())
    }

    #[test]
    fn number_cast_to_date_success() -> Result<(), FindItError> {
        let sql = "82800 as time";
        let eval = read_expr(sql)?;
        let file = Path::new("/no/such/file").to_path_buf();
        let wrapper = FileWrapper::new(file, 1);
        let value = eval.eval(&wrapper);

        assert_eq!(
            value,
            Value::Date(DateTime::from_timestamp(82800, 0).unwrap().into())
        );
        Ok(())
    }

    #[test]
    fn number_cast_to_date_fail() -> Result<(), FindItError> {
        let sql = "18446744073709551614 as time";
        let eval = read_expr(sql)?;
        let file = Path::new("/no/such/file").to_path_buf();
        let wrapper = FileWrapper::new(file, 1);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Empty);
        Ok(())
    }

    #[test]
    fn path_cast_to_date_fail() -> Result<(), FindItError> {
        let sql = "me as time";
        let eval = read_expr(sql)?;
        let file = Path::new("/no/such/file").to_path_buf();
        let wrapper = FileWrapper::new(file, 1);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Empty);
        Ok(())
    }

    #[test]
    fn path_cast_to_date_success() -> Result<(), FindItError> {
        let sql = "me as time";
        let eval = read_expr(sql)?;
        let file = Path::new("tests/test_cases/display/test_files/other-247.txt").to_path_buf();
        let access = file.metadata()?.accessed()?;
        let wrapper = FileWrapper::new(file, 1);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Date(access.into()));
        Ok(())
    }

    #[test]
    fn empty_cast_to_path() -> Result<(), FindItError> {
        let sql = "content as path";
        let eval = read_expr(sql)?;
        let file = Path::new("/no/such/file").to_path_buf();
        let wrapper = FileWrapper::new(file, 1);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Empty);
        Ok(())
    }

    #[test]
    fn bool_cast_to_path() -> Result<(), FindItError> {
        let sql = "true as path";
        let eval = read_expr(sql)?;
        let file = Path::new("/no/such/file").to_path_buf();
        let wrapper = FileWrapper::new(file, 1);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Empty);
        Ok(())
    }

    #[test]
    fn number_cast_to_path() -> Result<(), FindItError> {
        let sql = "10 as path";
        let eval = read_expr(sql)?;
        let file = Path::new("/no/such/file").to_path_buf();
        let wrapper = FileWrapper::new(file, 1);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Empty);
        Ok(())
    }

    #[test]
    fn date_cast_to_path() -> Result<(), FindItError> {
        let sql = "now() as path";
        let eval = read_expr(sql)?;
        let file = Path::new("/no/such/file").to_path_buf();
        let wrapper = FileWrapper::new(file, 1);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Empty);
        Ok(())
    }

    #[test]
    fn path_cast_to_path() -> Result<(), FindItError> {
        let sql = "me as path";
        let eval = read_expr(sql)?;
        let file = Path::new("/no/such/file");
        let wrapper = FileWrapper::new(file.to_path_buf(), 1);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Path(file.to_path_buf()));
        Ok(())
    }

    #[test]
    fn string_cast_to_path() -> Result<(), FindItError> {
        let sql = "\".\" as path";
        let eval = read_expr(sql)?;
        let file = Path::new("/no/such/file");
        let wrapper = FileWrapper::new(file.to_path_buf(), 1);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Path(Path::new(".").into()));
        Ok(())
    }

    #[test]
    fn test_cast_to_bool_type() -> Result<(), FindItError> {
        let sql = "true as bool";
        let eval = read_expr(sql)?;

        assert_eq!(eval.expected_type(), ValueType::Bool);
        Ok(())
    }

    #[test]
    fn test_cast_to_string_type() -> Result<(), FindItError> {
        let sql = "true as string";
        let eval = read_expr(sql)?;

        assert_eq!(eval.expected_type(), ValueType::String);
        Ok(())
    }

    #[test]
    fn test_cast_to_date_type() -> Result<(), FindItError> {
        let sql = "me as date";
        let eval = read_expr(sql)?;

        assert_eq!(eval.expected_type(), ValueType::Date);
        Ok(())
    }

    #[test]
    fn test_cast_to_path_type() -> Result<(), FindItError> {
        let sql = "me as file";
        let eval = read_expr(sql)?;

        assert_eq!(eval.expected_type(), ValueType::Path);
        Ok(())
    }

    #[test]
    fn test_cast_to_number_type() -> Result<(), FindItError> {
        let sql = "11 as number";
        let eval = read_expr(sql)?;

        assert_eq!(eval.expected_type(), ValueType::Number);
        Ok(())
    }

    #[test]
    fn test_cast_list_to_date() -> Result<(), FindItError> {
        let sql = "[1, 2] as date";
        let eval = read_expr(sql)?;

        let file = Path::new("/no/such/file");
        let wrapper = FileWrapper::new(file.to_path_buf(), 1);
        let value = eval.eval(&wrapper);

        assert_eq!(value, Value::Empty);
        Ok(())
    }
}
