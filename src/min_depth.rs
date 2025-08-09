use crate::{cli_args::CliArgs, file_wrapper::FileWrapper, output::build_output, walker::Walk};

struct MinDepth {
    min: usize,
    next: Box<dyn Walk>,
}

pub(crate) fn build_min(args: &CliArgs) -> Box<dyn Walk> {
    let next = build_output(args);
    let Some(min) = args.min_depth else {
        return next;
    };

    Box::new(MinDepth { min, next })
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
