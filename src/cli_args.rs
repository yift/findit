use std::path::PathBuf;

use clap::Parser;

/// Find files
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub(crate) struct CliArgs {
    /// Root to find from, default to current directory
    pub(crate) root: Option<PathBuf>,

    /// Which files to consider (by default, all of them)
    #[arg(short = 'w', long = "where", value_name = "FILTER")]
    pub(crate) filter: Vec<String>,

    /// How to order the files
    #[arg(short, long, value_name = "ORDER BY")]
    pub(crate) order_by: Option<String>,

    /// Max depth
    #[arg(short = 'x', long)]
    pub(crate) max_depth: Option<usize>,

    /// Min depth
    #[arg(short = 'n', long)]
    pub(crate) min_depth: Option<usize>,

    /// Limit the number of files
    #[arg(short, long)]
    pub(crate) limit: Option<usize>,

    /// What to execute on each file
    #[arg(short, long)]
    pub(crate) execute: Option<String>,

    /// What to display on each file (default will display the path)
    #[arg(short, long)]
    pub(crate) display: Option<String>,

    /// Start of string interpolation in the display or execute
    #[arg(long, default_value = "`")]
    pub(crate) interpolation_start: String,

    /// End of string interpolation in the display or execute
    #[arg(long, default_value = "`")]
    pub(crate) interpolation_end: String,

    /// Consider nodes before their parent
    #[arg(long, default_value_t = false)]
    pub(crate) node_first: bool,
}
