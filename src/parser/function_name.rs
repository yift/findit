use crate::parser::ast::function_name::{
    EnvFunctionName, FunctionName, StringFunctionName, TimeFunctionName,
};

impl FunctionName {
    pub(super) fn from_str(name: &str) -> Option<Self> {
        match name {
            "RAND" | "RANDOM" => Some(FunctionName::Env(EnvFunctionName::Rand)),
            "ENVIRONMENT" | "ENV" => Some(FunctionName::Env(EnvFunctionName::Env)),
            "COALESCE" => Some(FunctionName::Env(EnvFunctionName::Coalesce)),
            "TRIM" => Some(FunctionName::String(StringFunctionName::Trim)),
            "TRIM_HEAD" | "TRIMHEAD" => Some(FunctionName::String(StringFunctionName::TrimHead)),
            "TRIM_TAIL" | "TRIMTAIL" => Some(FunctionName::String(StringFunctionName::TrimTail)),
            "EXECUTE_OUTPUT" | "EXECUTEOUTPUT" | "EXECOUT" | "EXEC_OUT" => {
                Some(FunctionName::Env(EnvFunctionName::ExecOut))
            }
            "LENGTH" | "LEN" => Some(FunctionName::String(StringFunctionName::Length)),
            "UPPER" => Some(FunctionName::String(StringFunctionName::Upper)),
            "LOWER" => Some(FunctionName::String(StringFunctionName::Lower)),
            "NOW" => Some(FunctionName::Time(TimeFunctionName::Now)),
            _ => None,
        }
    }
}
