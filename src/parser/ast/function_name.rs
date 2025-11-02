#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum EnvFunctionName {
    Rand,
    Env,
    Coalesce,
    ExecOut,
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum TimeFunctionName {
    Now,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum FunctionName {
    Env(EnvFunctionName),
    Time(TimeFunctionName),
}
