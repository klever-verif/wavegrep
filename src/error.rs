#![allow(dead_code)]

use thiserror::Error;

#[derive(Debug, Error)]
pub enum WavepeekError {
    #[error("error: args: {0}")]
    Args(String),
    #[error("error: file: {0}")]
    File(String),
    #[error("error: unimplemented: {0}")]
    Unimplemented(&'static str),
}

impl WavepeekError {
    pub const fn exit_code(&self) -> u8 {
        match self {
            Self::File(_) => 2,
            Self::Args(_) | Self::Unimplemented(_) => 1,
        }
    }
}
