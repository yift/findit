use crate::{cli_args::CliArgs, file_wrapper::FileWrapper, walker::Walk};

#[derive(Debug)]
struct Limit {
    limit: usize,
    counter: usize,
}

impl Walk for Limit {
    fn step(&mut self, _file: &FileWrapper) {
        self.counter += 1;
    }
    fn enough(&self) -> bool {
        self.counter >= self.limit
    }
}
pub(crate) fn make_limit(args: &CliArgs) -> Option<Box<dyn Walk>> {
    let limit = args.limit?;
    Some(Box::new(Limit { limit, counter: 0 }))
}
