use crate::cli::when::WhenArgs;
use crate::engine::CommandResult;
use crate::error::WavepeekError;

pub fn run(_args: WhenArgs) -> Result<CommandResult, WavepeekError> {
    Err(WavepeekError::Unimplemented(
        "`when` command execution is not implemented yet",
    ))
}
