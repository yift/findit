use crate::parser::ast::function_name::{EnvFunctionName, FunctionName, TimeFunctionName};

impl FunctionName {
    pub(super) fn from_str(name: &str) -> Option<Self> {
        match name {
            "RAND" | "RANDOM" => Some(FunctionName::Env(EnvFunctionName::Rand)),
            "ENVIRONMENT" | "ENV" => Some(FunctionName::Env(EnvFunctionName::Env)),
            "COALESCE" => Some(FunctionName::Env(EnvFunctionName::Coalesce)),
            "EXECUTE_OUTPUT" | "EXECUTEOUTPUT" | "EXECOUT" | "EXEC_OUT" => {
                Some(FunctionName::Env(EnvFunctionName::ExecOut))
            }
            "NOW" => Some(FunctionName::Time(TimeFunctionName::Now)),
            _ => None,
        }
    }
}
