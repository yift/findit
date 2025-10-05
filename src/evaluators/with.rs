use crate::{
    errors::FindItError,
    evaluators::expr::{BindingsTypes, Evaluator, EvaluatorFactory},
    file_wrapper::FileWrapper,
    parser::ast::{expression::Expression, with::With as WithExpression},
    value::{Value, ValueType},
};

struct With {
    definition: Box<dyn Evaluator>,
    action: Box<dyn Evaluator>,
}
impl Evaluator for With {
    fn expected_type(&self) -> ValueType {
        self.action.expected_type()
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let value = self.definition.eval(file);
        let new_file = file.with_binding(value);
        self.action.eval(&new_file)
    }
}

impl EvaluatorFactory for WithExpression {
    fn build(&self, bindings: &BindingsTypes) -> Result<Box<dyn Evaluator>, FindItError> {
        build_with(&self.names, &self.action, bindings)
    }
}
fn build_with(
    names: &[(String, Box<Expression>)],
    action: &Expression,
    bindings: &BindingsTypes,
) -> Result<Box<dyn Evaluator>, FindItError> {
    let Some((first_name, expr)) = names.first() else {
        return action.build(bindings);
    };
    let definition = expr.build(bindings)?;
    let new_bindings = bindings.with(first_name, definition.expected_type());
    let action = build_with(&names[1..], action, &new_bindings)?;
    Ok(Box::new(With { definition, action }))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{
        errors::FindItError,
        evaluators::expr::read_expr,
        file_wrapper::FileWrapper,
        value::{Value, ValueType},
    };

    #[test]
    fn test_simple_with() -> Result<(), FindItError> {
        let expr = read_expr("with {one} as 1 do 1 + {one} + 10 end")?;
        let file = &FileWrapper::new(PathBuf::new(), 1);

        assert_eq!(expr.eval(file), Value::Number(12));

        Ok(())
    }

    #[test]
    fn test_multiple_with() -> Result<(), FindItError> {
        let expr = read_expr(
            "with {one} 1, {two} as 1 + {one}, {three} {two} + 1, {four} as {two} * {two} do {three} + {four} end",
        )?;
        let file = &FileWrapper::new(PathBuf::new(), 1);

        assert_eq!(expr.eval(file), Value::Number(7));

        Ok(())
    }

    #[test]
    fn test_multiple_overwrite() -> Result<(), FindItError> {
        let expr = read_expr(
            "with {one} 1, {two} {one} + {one}, {one} \"one\" do {one} + {two} as string end",
        )?;
        let file = &FileWrapper::new(PathBuf::new(), 1);

        assert_eq!(expr.eval(file), Value::String("one2".into()));

        Ok(())
    }

    #[test]
    fn test_expected_value() -> Result<(), FindItError> {
        let expr = read_expr("with {content} content do {content} end")?;

        assert_eq!(expr.expected_type(), ValueType::String);

        Ok(())
    }
    #[test]
    fn test_with_with_unknown_type() {
        let err = read_expr("with {one} as 1 do 1 + {two} + 10 end").err();

        assert!(err.is_some());
    }
}
