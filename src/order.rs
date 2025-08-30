use std::{cmp::Ordering, io::Write};

use crate::{
    cli_args::CliArgs,
    errors::FindItError,
    expr::{Evaluator, read_order_by},
    file_wrapper::FileWrapper,
    output::build_output,
    value::Value,
    walker::Walk,
};

pub(crate) enum OrderDirection {
    Asc,
    Desc,
}
pub(crate) struct OrderItem {
    pub(crate) direction: OrderDirection,
    pub(crate) evaluator: Box<dyn Evaluator>,
    pub(crate) nulls_first: bool,
}

impl OrderItem {
    fn compare(&self, left: &FileWrapper, right: &FileWrapper) -> Ordering {
        let left = self.evaluator.eval(left);
        let right = self.evaluator.eval(right);

        let ret = self.compare_as_is(&left, &right);
        match self.direction {
            OrderDirection::Asc => ret,
            OrderDirection::Desc => ret.reverse(),
        }
    }
    fn compare_as_is(&self, left: &Value, right: &Value) -> Ordering {
        if *left == Value::Empty {
            if *right == Value::Empty {
                Ordering::Equal
            } else if self.nulls_first {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        } else if *right == Value::Empty {
            if self.nulls_first {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        } else {
            left.cmp(right)
        }
    }
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
        self.items.sort_by(|left, right| {
            for item in &self.order {
                let order = item.compare(left, right);
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
