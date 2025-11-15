use crate::{
    errors::FindItError,
    evaluators::{
        expr::{BindingsTypes, Evaluator, EvaluatorFactory},
        extract::MeExtractor,
        method_invocation::{
            filter::new_filter,
            join::new_join,
            length::new_length,
            lines::new_lines,
            map::new_map,
            reverse::new_reverse,
            skip::new_skip,
            sort::{new_sort, new_sort_by},
            split::new_split,
            sum::new_sum,
            take::new_take,
            to_lower::new_to_lower,
            to_upper::new_to_upper,
            trim::{new_trim, new_trim_head, new_trim_tail},
            words::new_words,
        },
    },
    parser::ast::methods::{Method, MethodInvocation},
};

mod filter;
mod join;
mod lambda_builder;
mod length;
mod lines;
mod map;
mod reverse;
mod skip;
mod sort;
mod split;
mod sum;
mod take;
mod to_lower;
mod to_upper;
mod trim;
mod words;

impl EvaluatorFactory for MethodInvocation {
    fn build(&self, bindings: &BindingsTypes) -> Result<Box<dyn Evaluator>, FindItError> {
        let target = match &self.target {
            Some(target) => target.build(bindings)?,
            None => Box::new(MeExtractor {}),
        };
        match &self.method {
            Method::Length => new_length(target),

            Method::ToUpper => new_to_upper(target),
            Method::ToLower => new_to_lower(target),
            Method::Trim => new_trim(target),
            Method::TrimHead => new_trim_head(target),
            Method::TrimTail => new_trim_tail(target),
            Method::Reverse => new_reverse(target),
            Method::Map(lambda) => new_map(target, lambda, bindings),
            Method::Filter(lambda) => new_filter(target, lambda, bindings),
            Method::Sum => new_sum(target),
            Method::Sort => new_sort(target),
            Method::SortBy(lambda) => new_sort_by(target, lambda, bindings),
            Method::Skip(by) => new_skip(target, by, bindings),
            Method::Take(limit) => new_take(target, limit, bindings),
            Method::Join(delimiter) => new_join(target, delimiter, bindings),
            Method::Split(delimiter) => new_split(target, delimiter, bindings),
            Method::Lines => new_lines(target),
            Method::Words => new_words(target),
        }
    }
}
