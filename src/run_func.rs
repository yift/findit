use std::io::Write;

use crate::{cli_args::CliArgs, errors::FindItError, filter::make_filters, walker::Walker};

/// # Errors
///
/// Will return `Err` if anything goes wrong.
pub fn run<W: Write + 'static>(args: &CliArgs, writer: W) -> Result<(), FindItError> {
    let walker = Walker::try_from(args)?;
    let mut stepper = make_filters(args, writer)?;
    walker.walk(&mut stepper)?;
    Ok(())
}
