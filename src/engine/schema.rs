use crate::cli::schema::SchemaArgs;
use crate::engine::CommandResult;
use crate::error::WavepeekError;

pub fn run(_args: SchemaArgs) -> Result<CommandResult, WavepeekError> {
    Err(WavepeekError::Unimplemented(
        "`schema` command execution is not implemented yet",
    ))
}
