#![allow(dead_code)]

use serde::Serialize;

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Serialize)]
pub struct OutputEnvelope<T>
where
    T: Serialize,
{
    pub schema_version: u32,
    pub command: String,
    pub data: T,
    pub warnings: Vec<String>,
}

impl<T> OutputEnvelope<T>
where
    T: Serialize,
{
    pub fn new(command: impl Into<String>, data: T) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            command: command.into(),
            data,
            warnings: Vec::new(),
        }
    }
}
