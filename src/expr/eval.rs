use crate::error::WavepeekError;

use super::Expression;

pub fn eval(_expression: &Expression) -> Result<bool, WavepeekError> {
    Err(WavepeekError::Unimplemented(
        "expression evaluation is not implemented yet",
    ))
}
