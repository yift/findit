use crate::{
    errors::FindItError,
    evaluators::{
        expr::{BindingsTypes, Evaluator, EvaluatorFactory},
        extract::MeExtractor,
        method_invocation::{
            all::new_all,
            any::new_any,
            avg::new_avg,
            contains::new_contains,
            distinct::{new_distinct, new_distinct_by},
            filter::new_filter,
            first::new_first,
            flat_map::new_flat_map,
            group_by::new_group_by,
            has_prefix::new_has_prefix,
            has_suffix::new_has_suffix,
            index_of::new_index_of,
            join::new_join,
            last::new_last,
            length::new_length,
            lines::new_lines,
            map::new_map,
            max::new_max,
            min::new_min,
            remove_prefix::new_remove_prefix,
            remove_suffix::new_remove_suffix,
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

mod all;
mod any;
mod avg;
mod contains;
mod distinct;
mod enumerate;
mod filter;
mod first;
mod flat_map;
mod group_by;
mod has_prefix;
mod has_suffix;
mod index_of;
mod join;
mod lambda_builder;
mod last;
mod length;
mod lines;
mod map;
mod max;
mod min;
mod remove_prefix;
mod remove_suffix;
mod reverse;
mod skip;
mod sort;
mod split;
mod sum;
mod take;
mod to_lower;
mod to_upper;
mod trim;
mod walk;
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
            Method::Avg => new_avg(target),
            Method::Max => new_max(target),
            Method::Min => new_min(target),
            Method::Sort => new_sort(target),
            Method::Distinct => new_distinct(target),
            Method::DistinctBy(lambda) => new_distinct_by(target, lambda, bindings),
            Method::SortBy(lambda) => new_sort_by(target, lambda, bindings),
            Method::Skip(by) => new_skip(target, by, bindings),
            Method::Take(limit) => new_take(target, limit, bindings),
            Method::Join(delimiter) => new_join(target, delimiter, bindings),
            Method::Split(delimiter) => new_split(target, delimiter, bindings),
            Method::Lines => new_lines(target),
            Method::Words => new_words(target),
            Method::First => new_first(target),
            Method::Last => new_last(target),
            Method::Contains(item_to_find) => new_contains(target, item_to_find, bindings),
            Method::IndexOf(item_to_find) => new_index_of(target, item_to_find, bindings),
            Method::FlatMap(lambda) => new_flat_map(target, lambda, bindings),
            Method::All(lambda) => new_all(target, lambda, bindings),
            Method::Any(lambda) => new_any(target, lambda, bindings),
            Method::GroupBy(lambda) => new_group_by(target, lambda, bindings),
            Method::Enumerate => enumerate::new_enumerate(target),
            Method::Walk => walk::new_walker(target),
            Method::HasPrefix(prefix) => new_has_prefix(target, prefix, bindings),
            Method::HasSuffix(suffix) => new_has_suffix(target, suffix, bindings),
            Method::RemovePrefix(prefix) => new_remove_prefix(target, prefix, bindings),
            Method::RemoveSuffix(suffix) => new_remove_suffix(target, suffix, bindings),
        }
    }
}
