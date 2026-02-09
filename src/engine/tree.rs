use crate::cli::tree::TreeArgs;
use crate::engine::CommandResult;
use crate::error::WavepeekError;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TreeEntry {
    pub path: String,
    pub depth: usize,
}

pub fn run(_args: TreeArgs) -> Result<CommandResult, WavepeekError> {
    Err(WavepeekError::Unimplemented(
        "`tree` command execution is not implemented yet",
    ))
}
