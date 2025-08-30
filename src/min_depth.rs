use std::io::Write;

use crate::{
    cli_args::CliArgs, errors::FindItError, file_wrapper::FileWrapper, order::build_order_by,
    walker::Walk,
};

struct MinDepth {
    min: usize,
    next: Box<dyn Walk>,
}

pub(crate) fn build_min<W: Write + 'static>(
    args: &CliArgs,
    writer: W,
) -> Result<Box<dyn Walk>, FindItError> {
    let next = build_order_by(args, writer)?;
    let Some(min) = args.min_depth else {
        return Ok(next);
    };

    Ok(Box::new(MinDepth { min, next }))
}

impl Walk for MinDepth {
    fn step(&mut self, file: &FileWrapper) {
        if file.dept() >= self.min {
            self.next.step(file);
        }
    }
    fn enough(&self) -> bool {
        self.next.enough()
    }
}
