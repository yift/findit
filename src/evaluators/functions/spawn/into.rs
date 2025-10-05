use crate::{
    errors::FindItError,
    evaluators::{
        expr::{BindingsTypes, Evaluator, EvaluatorFactory},
        functions::spawn::{
            exec::{ExecType, build_exec},
            execute::Executor,
            fire::build_fire,
        },
    },
    parser::ast::execute::SpawnOrExecute,
    value::ValueType,
};

impl EvaluatorFactory for SpawnOrExecute {
    fn build(&self, bindings: &BindingsTypes) -> Result<Box<dyn Evaluator>, FindItError> {
        let exec = self.bin.build(bindings)?;
        if exec.expected_type() != ValueType::String && exec.expected_type() != ValueType::Path {
            return Err(FindItError::BadExpression(
                "Can only execute string or files.".into(),
            ));
        }

        let mut args = vec![];
        for arg in &self.args {
            args.push(arg.build(bindings)?);
        }

        let (exec_type, into) = match &self.into {
            Some(into) => {
                let into = into.build(bindings)?;
                if into.expected_type() != ValueType::String
                    && into.expected_type() != ValueType::Path
                {
                    return Err(FindItError::BadExpression(
                        "Can only fire into string or file.".into(),
                    ));
                }

                (ExecType::IntoStatus, Some(into))
            }
            None => (ExecType::Status, None),
        };

        let executor = Executor::new(exec, args, into);
        if self.spawn {
            Ok(build_fire(executor))
        } else {
            Ok(build_exec(executor, exec_type))
        }
    }
}
