use crate::error::WavepeekError;

use super::Expression;

pub fn parse(source: &str) -> Result<Expression, WavepeekError> {
    if source.trim().is_empty() {
        return Err(WavepeekError::Args(
            "--cond expression cannot be empty".to_owned(),
        ));
    }

    Ok(Expression {
        source: source.to_owned(),
    })
}
