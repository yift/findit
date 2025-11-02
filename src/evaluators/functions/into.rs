use std::collections::VecDeque;

use crate::{
    errors::FindItError,
    evaluators::{
        expr::{BindingsTypes, Evaluator, EvaluatorFactory},
        functions::{
            conditional::{coalesce::build_coalesce, random::build_rand},
            env::build_env,
            spawn::exec::build_capture_output_exec,
            time::now::build_now,
        },
    },
    parser::ast::{
        function::Function,
        function_name::{EnvFunctionName, FunctionName, TimeFunctionName},
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
