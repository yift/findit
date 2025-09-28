#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum StringFunctionName {
    Trim,
    TrimHead,
    TrimTail,
    Length,
    Upper,
    Lower,
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
