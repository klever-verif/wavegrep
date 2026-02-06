#![allow(dead_code)]

use std::path::Path;

use crate::error::WavepeekError;

#[derive(Debug, Default)]
pub struct Waveform;

impl Waveform {
    pub fn open(_path: &Path) -> Result<Self, WavepeekError> {
        Err(WavepeekError::Unimplemented(
            "waveform loading is not implemented yet",
        ))
    }
}
