use std::rc::Rc;

use crate::{
    class_type::{Class, ClassType},
    errors::FindItError,
    evaluators::expr::{BindingsTypes, Evaluator, EvaluatorFactory},
    file_wrapper::FileWrapper,
    parser::ast::class::ClassDefinition,
    value::{Value, ValueType},
};

struct ClassBuilder {
    cls: Rc<ClassType>,
    fields: Vec<Box<dyn Evaluator>>,
}
impl Evaluator for ClassBuilder {
    fn expected_type(&self) -> ValueType {
        ValueType::Class(self.cls.clone())
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        let fields = self.fields.iter().map(|f| f.eval(file)).collect();
        let cls = Class::new(&self.cls, fields);
        Value::Class(cls)
    }
}

impl EvaluatorFactory for ClassDefinition {
    fn build(&self, bindings: &BindingsTypes) -> Result<Box<dyn Evaluator>, FindItError> {
        let mut fields = Vec::new();
        let mut defs = Vec::new();
        for field in &self.fields {
            let access = field.value.build(bindings)?;
            defs.push((field.name.clone(), access.expected_type()));
            fields.push(access);
        }

        let cls = Rc::new(ClassType::new(&defs));
        Ok(Box::new(ClassBuilder { cls, fields }))
    }
}

#[cfg(test)]
mod tests {

    use std::{path::Path, rc::Rc};

    use crate::{
        class_type::{Class, ClassType},
        errors::FindItError,
        evaluators::expr::read_expr,
        file_wrapper::FileWrapper,
        value::{Value, ValueType},
    };

    #[test]
    fn simple_happy_path() -> Result<(), FindItError> {
        let expr = read_expr("{:one 1, :two \"2\"}")?;
        let file = Path::new("/tmp");
        let wrapper = FileWrapper::new(file.to_path_buf(), 1);

        let value = expr.eval(&wrapper);

        let tp = ClassType::new(&[
            ("one".into(), ValueType::Number),
            ("two".into(), ValueType::String),
        ]);
        let cls = Class::new(
            &Rc::new(tp),
            vec![Value::Number(1), Value::String("2".into())],
        );
        assert_eq!(value, Value::Class(cls));

        Ok(())
    }
}
