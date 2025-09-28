#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum StringFunctionName {
    Trim,
    TrimHead,
    TrimTail,
    Length,
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
