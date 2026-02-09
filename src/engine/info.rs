use crate::cli::info::InfoArgs;
use crate::engine::CommandResult;
use crate::error::WavepeekError;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InfoData {
    pub time_unit: String,
    pub time_precision: String,
    pub time_start: String,
    pub time_end: String,
}

pub fn run(_args: InfoArgs) -> Result<CommandResult, WavepeekError> {
    Err(WavepeekError::Unimplemented(
        "`info` command execution is not implemented yet",
    ))
}
