use crate::cli::at::AtArgs;
use crate::engine::CommandResult;
use crate::error::WavepeekError;

pub fn run(_args: AtArgs) -> Result<CommandResult, WavepeekError> {
    Err(WavepeekError::Unimplemented(
        "`at` command execution is not implemented yet",
    ))
}
