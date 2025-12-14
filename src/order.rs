use std::{cmp::Ordering, collections::HashMap, io::Write};

use crate::{
    cli_args::CliArgs,
    errors::FindItError,
    evaluators::expr::{Evaluator, read_order_by},
    file_wrapper::FileWrapper,
    output::build_output,
    walker::Walk,
};

pub(crate) enum OrderDirection {
    Asc,
    Desc,
}
pub(crate) struct OrderItem {
    pub(crate) direction: OrderDirection,
    pub(crate) evaluator: Box<dyn Evaluator>,
}

struct OrderBy {
    next: Box<dyn Walk>,
    order: Vec<OrderItem>,
    items: Vec<FileWrapper>,
}

impl Walk for OrderBy {
    fn enough(&self) -> bool {
        false
    }
    fn step(&mut self, file: &FileWrapper) {
        self.items.push(file.clone());
    }
}
impl Drop for OrderBy {
    fn drop(&mut self) {
        let mut cache = HashMap::new();
        self.items.sort_by(|left, right| {
            for (index, item) in self.order.iter().enumerate() {
                let left = cache
                    .entry((index, left.path().to_path_buf()))
                    .or_insert_with(|| item.evaluator.eval(left))
                    .clone();
                let right = cache
                    .entry((index, right.path().to_path_buf()))
                    .or_insert_with(|| item.evaluator.eval(right))
                    .clone();
                let cmp = left.cmp(&right);
                let order = match item.direction {
                    OrderDirection::Asc => cmp,
                    OrderDirection::Desc => cmp.reverse(),
                };
                if order != Ordering::Equal {
                    return order;
                }
            }
            Ordering::Equal
        });
        for file in &self.items {
            if self.next.enough() {
                return;
            }
            self.next.step(file);
        }
    }
}
pub(crate) fn build_order_by<W: Write + 'static>(
    args: &CliArgs,
    writer: W,
) -> Result<Box<dyn Walk>, FindItError> {
    let next = build_output(args, writer)?;
    let Some(order) = &args.order_by else {
        return Ok(next);
    };
    let order = read_order_by(order)?;
    Ok(Box::new(OrderBy {
        next,
        order,
        items: vec![],
    }))
}
