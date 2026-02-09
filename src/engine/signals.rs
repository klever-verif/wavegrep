use crate::cli::signals::SignalsArgs;
use crate::engine::CommandResult;
use crate::error::WavepeekError;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SignalEntry {
    pub name: String,
    pub path: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
}

pub fn run(_args: SignalsArgs) -> Result<CommandResult, WavepeekError> {
    Err(WavepeekError::Unimplemented(
        "`signals` command execution is not implemented yet",
    ))
}
