use std::io::Write;

use crate::evaluators::expr::{Evaluator, read_expr};
use crate::value::Value;
use crate::{
    cli_args::CliArgs, errors::FindItError, file_wrapper::FileWrapper, min_depth::build_min,
    walker::Walk,
};
struct Filter {
    next: Box<dyn Walk>,
    expr: Box<dyn Evaluator>,
}
impl Walk for Filter {
    fn enough(&self) -> bool {
        self.next.enough()
    }
    fn step(&mut self, file: &FileWrapper) {
        if self.expr.eval(file) == Value::Bool(true) {
            self.next.step(file);
        }
    }
}
pub(crate) fn make_filters<W: Write + 'static>(
    args: &CliArgs,
    writer: W,
) -> Result<Box<dyn Walk>, FindItError> {
    let mut last = build_min(args, writer)?;
    for sql in &args.filter {
        let expr = read_expr(sql)?;
        last = Box::new(Filter { expr, next: last });
    }

    Ok(last)
}
