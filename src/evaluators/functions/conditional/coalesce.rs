use std::collections::VecDeque;

use crate::{
    errors::FindItError,
    evaluators::expr::Evaluator,
    file_wrapper::FileWrapper,
    value::{Value, ValueType},
};

struct Coalesce {
    args: VecDeque<Box<dyn Evaluator>>,
    value_type: ValueType,
}

impl Evaluator for Coalesce {
    fn eval(&self, file: &FileWrapper) -> Value {
        for arg in &self.args {
            let value = arg.eval(file);
            if value != Value::Empty {
                return value;
            }
        }
        Value::Empty
    }
    fn expected_type(&self) -> ValueType {
        self.value_type.clone()
    }
}

pub(crate) fn build_coalesce(
    args: VecDeque<Box<dyn Evaluator>>,
) -> Result<Box<dyn Evaluator>, FindItError> {
    let Some(value_type) = args.iter().next().map(|e| e.expected_type()) else {
        return Err(FindItError::BadExpression(
            "coalesce must have arguments.".into(),
        ));
    };
    for a in &args {
        if a.expected_type() != value_type {
            return Err(FindItError::BadExpression(
                "All the coalesce arguments must have the same type.".into(),
            ));
        }
    }
    Ok(Box::new(Coalesce { args, value_type }))
}

#[cfg(test)]
mod tests {

    use std::{fs, path::Path};

    use crate::{
        errors::FindItError,
        evaluators::expr::read_expr,
        file_wrapper::FileWrapper,
        value::{Value, ValueType},
    };

    #[test]
    fn coalesce_with_no_args() {
        let sql = "Coalesce()";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }

    #[test]
    fn coalesce_with_args_with_different_type() {
        let sql = "Coalesce(1, true)";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }

    #[test]
    fn coalesce_return_value() -> Result<(), FindItError> {
        let sql = "Coalesce(1)";
        let eval = read_expr(sql)?;

        assert_eq!(eval.expected_type(), ValueType::Number);

        Ok(())
    }

    #[test]
    fn coalesce_return_first_non_empty_value() -> Result<(), FindItError> {
        let sql = "Coalesce(parent.content, parent.parent.content, content, \"text\")";
        let eval = read_expr(sql)?;
        let file = Path::new("tests/test_cases/display/test_files/week-362.txt");
        let expected = fs::read_to_string(file)?;
        let wrapper = FileWrapper::new(file.to_path_buf(), 1);

        assert_eq!(eval.eval(&wrapper), Value::String(expected));

        Ok(())
    }

    #[test]
    fn coalesce_return_empty_for_empty_value() -> Result<(), FindItError> {
        let sql = "Coalesce(parent.content, parent.parent.content, content)";
        let eval = read_expr(sql)?;
        let file = Path::new("no/such/file.txt");
        let wrapper = FileWrapper::new(file.to_path_buf(), 1);

        assert_eq!(eval.eval(&wrapper), Value::Empty);

        Ok(())
    }
}
