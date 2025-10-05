use crate::{
    errors::FindItError,
    evaluators::expr::{BindingsTypes, Evaluator, EvaluatorFactory},
    file_wrapper::FileWrapper,
    parser::ast::binding::Binding,
    value::{Value, ValueType},
};

struct BindingReplacement {
    index: usize,
    value_type: ValueType,
}
impl Evaluator for BindingReplacement {
    fn expected_type(&self) -> ValueType {
        self.value_type
    }
    fn eval(&self, file: &FileWrapper) -> Value {
        file.get_binding(self.index)
    }
}

impl EvaluatorFactory for Binding {
    fn build(&self, bindings: &BindingsTypes) -> Result<Box<dyn Evaluator>, FindItError> {
        let (index, value_type) = bindings.get(&self.name)?;
        Ok(Box::new(BindingReplacement {
            index: *index,
            value_type: *value_type,
        }))
    }
}
