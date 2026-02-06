use std::path::PathBuf;

use clap::Args;

#[derive(Debug, Args)]
pub struct AtArgs {
    /// Path to VCD/FST waveform file
    #[arg(long, value_name = "FILE")]
    pub waves: PathBuf,
    /// Time point with units (for example, 1337ns)
    #[arg(long)]
    pub time: String,
    /// Scope for short signal names
    #[arg(long)]
    pub scope: Option<String>,
    /// Comma-separated signal names
    #[arg(long, value_delimiter = ',', num_args = 1..)]
    pub signals: Vec<String>,
    /// Human-friendly output mode
    #[arg(long)]
    pub human: bool,
}
