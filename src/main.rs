#![deny(warnings)]

use std::io::stdout;

use clap::Parser;
use findit_cli::{cli_args::CliArgs, run_func::run};

fn main() {
    let args = CliArgs::parse();
    if let Err(e) = run(&args, stdout()) {
        eprintln!("{e}");
        std::process::exit(1)
    }
}
