use crate::{cli_args::CliArgs, file_wrapper::FileWrapper, limit::make_limit, walker::Walk};

struct Output {
    next: Option<Box<dyn Walk>>,
}

pub(crate) fn build_output(args: &CliArgs) -> Box<dyn Walk> {
    let next = make_limit(args);

    Box::new(Output { next })
}

impl Walk for Output {
    fn step(&mut self, file: &FileWrapper) {
        println!("{}", file);
        if let Some(next) = self.next.as_deref_mut() {
            next.step(file);
        }
    }
    fn enough(&self) -> bool {
        if let Some(next) = self.next.as_deref() {
            next.enough()
        } else {
            false
        }
    }
}
