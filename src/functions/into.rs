use std::collections::VecDeque;

use sqlparser::ast::{
    Function, FunctionArg, FunctionArgExpr, FunctionArgumentList, FunctionArguments,
};

use crate::{
    errors::FindItError,
    expr::{Evaluator, get_eval},
    functions::{
        conditional::{if_func::build_if, random::build_rand},
        spawn::{
            exec::{ExecType, build_exec},
            fire::build_fire,
        },
    },
};

pub(crate) fn new_function(function: &Function) -> Result<Box<dyn Evaluator>, FindItError> {
    if !function.within_group.is_empty() {
        return Err(FindItError::BadExpression("WITHIN GROUP".into()));
    }
    if function.over.is_some() {
        return Err(FindItError::BadExpression("OVER".into()));
    }
    if function.null_treatment.is_some() {
        return Err(FindItError::BadExpression("IGNORE/RESPECT NULLS".into()));
    }
    if function.filter.is_some() {
        return Err(FindItError::BadExpression("FILTER".into()));
    }
    if function.parameters != FunctionArguments::None {
        return Err(FindItError::BadExpression("function parameters".into()));
    }

    let name = function.name.to_string().to_uppercase();
    let args = match &function.args {
        FunctionArguments::List(lst) => build_args(lst)?,
        FunctionArguments::None => VecDeque::new(),
        FunctionArguments::Subquery(_) => {
            return Err(FindItError::BadExpression("function with sub query".into()));
        }
    };

    build_function(&name, args)
}

fn build_args(lst: &FunctionArgumentList) -> Result<VecDeque<Box<dyn Evaluator>>, FindItError> {
    if lst.duplicate_treatment.is_some() {
        return Err(FindItError::BadExpression("duplicate treatment".into()));
    }
    if !lst.clauses.is_empty() {
        return Err(FindItError::BadExpression("additional clauses".into()));
    }
    let mut ret = VecDeque::new();
    for arg in &lst.args {
        let FunctionArg::Unnamed(FunctionArgExpr::Expr(arg)) = arg else {
            return Err(FindItError::BadExpression("wildcard argument".into()));
        };
        ret.push_back(get_eval(arg)?);
    }

    Ok(ret)
}

fn build_function(
    name: &str,
    args: VecDeque<Box<dyn Evaluator>>,
) -> Result<Box<dyn Evaluator>, FindItError> {
    match name {
        "IF" => build_if(args),
        "RAND" | "RANDOM" => build_rand(&args),
        "FIRE" => build_fire(args, false),
        "FIRE_INTO" => build_fire(args, true),
        "EXEC" => build_exec(args, ExecType::Status),
        "EXEC_INTO" => build_exec(args, ExecType::IntoStatus),
        "EXEC_OUT" => build_exec(args, ExecType::CaptureOutput),
        _ => Err(FindItError::BadExpression(format!(
            "Function {name} is not supported."
        ))),
    }
}

#[cfg(test)]
mod tests {
    use crate::expr::read_expr;

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
