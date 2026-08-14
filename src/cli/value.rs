use std::path::PathBuf;

use clap::Args;

#[derive(Debug, Args)]
pub struct ValueArgs {
    /// Path to VCD/FST/FSDB waveform file
    #[arg(long, value_name = "FILE", help_heading = "Input options")]
    pub waves: PathBuf,
    /// Time point(s) with explicit units (e.g. 1337ns or 10ns,20ns)
    #[arg(long, help_heading = "Selection options")]
    pub at: String,
    /// Canonical scope path for relative or in-scope canonical signal names
    #[arg(long, help_heading = "Selection options")]
    pub scope: Option<String>,
    /// Comma-separated signal paths or flat [msb:lsb] projections
    #[arg(
        long,
        value_delimiter = ',',
        num_args = 1..,
        required = true,
        help_heading = "Selection options"
    )]
    pub signals: Vec<String>,
    /// Show canonical signal paths
    #[arg(long, help_heading = "Output options")]
    pub abs: bool,
    /// Machine-readable JSON output
    #[arg(long, help_heading = "Output options")]
    pub json: bool,
    /// Stream newline-delimited JSON output
    #[arg(long, conflicts_with = "json", help_heading = "Output options")]
    pub jsonl: bool,
}
