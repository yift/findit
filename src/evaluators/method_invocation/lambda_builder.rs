use crate::{
    errors::FindItError,
    evaluators::expr::{BindingsTypes, Evaluator, EvaluatorFactory},
    parser::ast::methods::LambdaFunction,
    value::ValueType,
};

impl LambdaFunction {
    pub(super) fn build(
        &self,
        bindings: &BindingsTypes,
        items_types: &ValueType,
    ) -> Result<Box<dyn Evaluator>, FindItError> {
        let new_bindings = bindings.with(&self.parameter, items_types.clone());

        self.body.build(&new_bindings)
    }
}
