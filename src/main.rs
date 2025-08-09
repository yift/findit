//#![deny(warnings)]

mod cli_args;
mod errors;
mod file_wrapper;
mod limit;
mod min_depth;
mod output;
mod walker;

use crate::{cli_args::CliArgs, errors::FindItError, min_depth::build_min, walker::Walker};
use clap::Parser;

fn main() -> Result<(), FindItError> {
    let args = CliArgs::parse();
    let walker = Walker::try_from(&args)?;
    let mut stepper = build_min(&args);
    walker.walk(&mut stepper)?;
    Ok(())
}
