//#![deny(warnings)]

mod cli_args;
mod errors;
mod expr;
mod extract;
mod file_wrapper;
mod filter;
mod limit;
mod min_depth;
mod output;
mod value;
mod walker;

use crate::{cli_args::CliArgs, errors::FindItError, filter::make_filters, walker::Walker};
use clap::Parser;

fn main() -> Result<(), FindItError> {
    let args = CliArgs::parse();
    let walker = Walker::try_from(&args)?;
    let mut stepper = make_filters(&args)?;
    walker.walk(&mut stepper)?;
    Ok(())
}
