use std::io::Write;

use crate::{
    cli_args::CliArgs, errors::FindItError, filter::make_filters, quick_ref::show_syntax_help,
    walker::Walker,
};

/// # Errors
///
/// Will return `Err` if anything goes wrong.
pub fn run<W: Write + 'static>(args: &CliArgs, writer: W) -> Result<(), FindItError> {
    if args.help_syntax {
        show_syntax_help();
    } else {
        let walker = Walker::try_from(args)?;
        let mut stepper = make_filters(args, writer)?;
        walker.walk(&mut stepper)?;
    }
    Ok(())
}
