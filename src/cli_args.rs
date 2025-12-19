use std::path::PathBuf;

use clap::Parser;

/// Find files using powerful filtering expressions
#[derive(Parser, Debug)]
#[command(version, about, version, long_about = None)]
pub struct CliArgs {
    /// Root directory to search (default: current directory)
    pub(crate) root: Option<PathBuf>,

    /// Filter which files to display using an expression
    ///
    /// Examples:
    ///   -w 'size > 1024'
    ///   -w 'extension = "rs" AND NOT content.contains("test")'
    #[arg(
        short = 'w',
        long = "where",
        value_name = "FILTER",
        visible_alias = "filter",
        help_heading = "Filtering Options"
    )]
    pub(crate) filter: Vec<String>,

    /// Sort results by an expression (add DESC for descending order)
    ///
    /// Examples:
    ///   -o 'size DESC'
    ///   -o 'modified, name'
    #[arg(
        short,
        long,
        value_name = "ORDER BY",
        visible_alias = "sort",
        visible_alias = "order",
        visible_alias = "sort-by",
        help_heading = "Output Ordering"
    )]
    pub(crate) order_by: Option<String>,

    /// Maximum depth to recurse into directories
    #[arg(short = 'x', long, help_heading = "Filtering Options")]
    pub(crate) max_depth: Option<usize>,

    /// Minimum depth to include files (0 = root level)
    #[arg(short = 'n', long, help_heading = "Filtering Options")]
    pub(crate) min_depth: Option<usize>,

    /// Maximum number of results to display
    #[arg(short, long, help_heading = "Filtering Options")]
    pub(crate) limit: Option<usize>,

    /// Custom output format using expressions in backticks (or `--interpolation-start` and `--interpolation-end`)
    ///
    /// Example:
    ///   -d 'File: `name`, Size: `size` bytes'
    #[arg(
        short,
        long,
        visible_alias = "show",
        visible_alias = "print",
        value_name = "DISPLAY",
        help_heading = "Output Formatting"
    )]
    pub(crate) display: Option<String>,

    /// Start marker for expressions in display format
    #[arg(
        long,
        default_value = "`",
        visible_alias = "expr-start",
        help_heading = "Output Formatting"
    )]
    pub(crate) interpolation_start: String,

    /// End marker for expressions in display format
    #[arg(
        long,
        default_value = "`",
        visible_alias = "expr-end",
        help_heading = "Output Formatting"
    )]
    pub(crate) interpolation_end: String,

    /// Process files before their parent directories
    #[arg(
        long,
        default_value_t = false,
        visible_alias = "files-first",
        visible_alias = "depth-first",
        help_heading = "Output Ordering"
    )]
    pub(crate) node_first: bool,

    /// Write debug information to a file
    #[arg(
        long,
        value_name = "DEBUG_FILE",
        visible_alias = "debug-log",
        help_heading = "Developer Options"
    )]
    pub(crate) debug_output_file: Option<PathBuf>,

    /// Show syntax help and examples
    #[arg(long, help_heading = "Developer Options")]
    pub(crate) help_syntax: bool,
}
