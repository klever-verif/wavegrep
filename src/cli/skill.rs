use std::path::PathBuf;

use clap::Args;

#[derive(Debug, Args)]
pub struct SkillArgs {
    /// New or empty destination directory
    pub directory: PathBuf,
}
