use clap::Args;

#[derive(Debug, Args)]
pub struct SchemaArgs {
    /// Human-friendly output mode
    #[arg(long)]
    pub human: bool,
}
