use crate::{
    errors::FindItError,
    expr::{Evaluator, get_eval},
    functions::spawn::{
        exec::{ExecType, build_exec},
        execute::Executor,
        fire::build_fire,
    },
    parser::ast::execute::SpawnOrExecute,
    value::ValueType,
};

impl TryFrom<&SpawnOrExecute> for Box<dyn Evaluator> {
    type Error = FindItError;
    fn try_from(value: &SpawnOrExecute) -> Result<Self, Self::Error> {
        let exec = get_eval(&value.bin)?;
        if exec.expected_type() != ValueType::String && exec.expected_type() != ValueType::Path {
            return Err(FindItError::BadExpression(
                "Can only execute string or files.".into(),
            ));
        }

        let mut args = vec![];
        for arg in &value.args {
            args.push(get_eval(arg)?);
        }

        let (exec_type, into) = match &value.into {
            Some(into) => {
                let into = get_eval(into)?;
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
        if value.spawn {
            Ok(build_fire(executor))
        } else {
            Ok(build_exec(executor, exec_type))
        }
    }
}
