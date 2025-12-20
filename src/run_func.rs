use std::io::Write;

use crate::{
    cli_args::CliArgs,
    errors::FindItError,
    filter::make_filters,
    quick_ref::Executor,
    quick_ref::Pager,
    quick_ref::show_syntax_help,
    quick_ref::{default_executor as executor, default_pager as pager},
    walker::Walker,
};

/// # Errors
///
/// Will return `Err` if anything goes wrong.
pub fn run<W: Write + 'static>(args: &CliArgs, writer: W) -> Result<(), FindItError> {
    run_with_pager_and_executor(args, writer, pager(), executor())
}
fn run_with_pager_and_executor<W: Write + 'static>(
    args: &CliArgs,
    writer: W,
    pager: impl Pager,
    executor: impl Executor,
) -> Result<(), FindItError> {
    if args.help_syntax {
        show_syntax_help(pager, executor);
    } else {
        let walker = Walker::try_from(args)?;
        let mut stepper = make_filters(args, writer)?;
        walker.walk(&mut stepper)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{os::unix::process::ExitStatusExt, process::ExitStatus};

    use clap::Parser;

    use crate::errors::FindItError;

    use super::*;

    #[test]
    fn run_with_pager_and_executor_will_not_crash_when_help_syntax_is_called()
    -> Result<(), FindItError> {
        struct TestPager;
        impl Pager for TestPager {
            fn pager(&self) -> String {
                "cat".to_string()
            }
        }
        struct TestExecutor;
        impl Executor for TestExecutor {
            fn spawn(&self, _: &str, _: &[&str], _: &[u8]) -> Result<(), FindItError> {
                Err(FindItError::PagerFailed(ExitStatus::from_raw(0)))
            }
        }
        let args = CliArgs::parse_from("findit --help-syntax /no/such/dir".split_whitespace());
        let writer = vec![];

        run_with_pager_and_executor(&args, writer, TestPager, TestExecutor)
    }
}
