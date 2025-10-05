use std::collections::VecDeque;

use crate::{
    errors::FindItError,
    evaluators::{
        expr::{BindingsTypes, Evaluator, EvaluatorFactory},
        functions::{
            conditional::{coalesce::build_coalesce, random::build_rand},
            env::build_env,
            spawn::exec::build_capture_output_exec,
            string_functions::{TrimWhere, new_length, new_lower, new_trim, new_upper},
            time::now::build_now,
        },
    },
    parser::ast::{
        function::Function,
        function_name::{EnvFunctionName, FunctionName, StringFunctionName, TimeFunctionName},
    },
};

impl EvaluatorFactory for Function {
    fn build(&self, bindings: &BindingsTypes) -> Result<Box<dyn Evaluator>, FindItError> {
        let mut args = VecDeque::new();

        for expr in &self.args {
            let eval = expr.build(bindings)?;
            args.push_back(eval);
        }
        match &self.name {
            FunctionName::Env(env) => new_env_function(env, args),
            FunctionName::String(string) => new_string_function(string, args),
            FunctionName::Time(time) => new_time_function(time, args),
        }
    }
}

fn new_env_function(
    name: &EnvFunctionName,
    args: VecDeque<Box<dyn Evaluator>>,
) -> Result<Box<dyn Evaluator>, FindItError> {
    match name {
        EnvFunctionName::Rand => build_rand(args),
        EnvFunctionName::Coalesce => build_coalesce(args),
        EnvFunctionName::Env => build_env(args),
        EnvFunctionName::ExecOut => build_capture_output_exec(args),
    }
}
fn new_string_function(
    name: &StringFunctionName,
    args: VecDeque<Box<dyn Evaluator>>,
) -> Result<Box<dyn Evaluator>, FindItError> {
    match name {
        StringFunctionName::Trim => new_trim(args, TrimWhere::Both),
        StringFunctionName::TrimHead => new_trim(args, TrimWhere::Head),
        StringFunctionName::TrimTail => new_trim(args, TrimWhere::Tail),
        StringFunctionName::Length => new_length(args),
        StringFunctionName::Lower => new_lower(args),
        StringFunctionName::Upper => new_upper(args),
    }
}

fn new_time_function(
    name: &TimeFunctionName,
    args: VecDeque<Box<dyn Evaluator>>,
) -> Result<Box<dyn Evaluator>, FindItError> {
    match name {
        TimeFunctionName::Now => build_now(args),
    }
}

#[cfg(test)]
mod tests {
    use crate::evaluators::expr::read_expr;

    #[test]
    fn test_function_within_group() {
        let sql = "percentile_disc(122) WITHIN GROUP (ORDER BY temperature)";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }

    #[test]
    fn test_function_respect_nulls() {
        let sql = "if(TRUE, 1, 2) RESPECT NULLS";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }

    #[test]
    fn test_function_filter() {
        let sql = "if(TRUE, 1, 2) FILTER (WHERE x > 5)";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }

    #[test]
    fn test_function_parameters() {
        let sql = "HISTOGRAM(0.5, 0.6)(x, y)";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }

    #[test]
    fn test_function_duplicate_treatment() {
        let sql = "IF(DISTINCT TRUE, 2, 3)";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }

    #[test]
    fn test_function_wildcard_argument() {
        let sql = "IF(*, 2, 3)";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }

    #[test]
    fn test_function_unknown() {
        let sql = "NOP(3, 1)";
        let err = read_expr(sql).err();

        assert!(err.is_some());
    }
}
