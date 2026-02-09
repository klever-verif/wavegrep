use crate::cli::changes::ChangesArgs;
use crate::engine::CommandResult;
use crate::error::WavepeekError;

pub fn run(_args: ChangesArgs) -> Result<CommandResult, WavepeekError> {
    Err(WavepeekError::Unimplemented(
        "`changes` command execution is not implemented yet",
    ))
}
