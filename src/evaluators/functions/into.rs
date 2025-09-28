use std::collections::VecDeque;

use crate::{
    errors::FindItError,
    evaluators::{
        expr::{Evaluator, get_eval},
        functions::{
            conditional::{coalesce::build_coalesce, random::build_rand},
            env::build_env,
            spawn::exec::build_capture_output_exec,
            string_functions::{TrimWhere, new_length, new_trim},
        },
    },
    parser::ast::{
        function::Function,
        function_name::{EnvFunctionName, FunctionName, StringFunctionName},
    },
};

impl TryFrom<&Function> for Box<dyn Evaluator> {
    type Error = FindItError;
    fn try_from(function: &Function) -> Result<Self, Self::Error> {
        let mut args = VecDeque::new();

        for expr in &function.args {
            let eval = get_eval(expr)?;
            args.push_back(eval);
        }
        match &function.name {
            FunctionName::Env(env) => new_env_function(env, args),
            FunctionName::String(string) => new_string_function(string, args),
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
