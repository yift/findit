use std::fmt::Display;

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum StringFunctionName {
    Trim,
    TrimHead,
    TrimTail,
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum EnvFunctionName {
    Rand,
    Env,
    Coalesce,
    ExecOut,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum FunctionName {
    String(StringFunctionName),
    Env(EnvFunctionName),
}

impl Display for StringFunctionName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StringFunctionName::Trim => write!(f, "Trim"),
            StringFunctionName::TrimHead => write!(f, "TrimHead"),
            StringFunctionName::TrimTail => write!(f, "TrimTail"),
        }
    }
}

impl Display for EnvFunctionName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnvFunctionName::Rand => write!(f, "Rand"),
            EnvFunctionName::Env => write!(f, "Env"),
            EnvFunctionName::Coalesce => write!(f, "Coalesce"),
            EnvFunctionName::ExecOut => write!(f, "ExecOut"),
        }
    }
}
impl Display for FunctionName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FunctionName::Env(e) => write!(f, "{}", e),
            FunctionName::String(e) => write!(f, "{}", e),
        }
    }
}

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
            _ => None,
        }
    }
}
