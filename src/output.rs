use serde::Serialize;

use crate::engine::{CommandData, CommandResult};
use crate::error::WavepeekError;

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
    pub fn with_warnings(command: impl Into<String>, data: T, warnings: Vec<String>) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            command: command.into(),
            data,
            warnings,
        }
    }
}

pub fn write(result: CommandResult) -> Result<(), WavepeekError> {
    if result.human {
        let output = render_human(&result.data);
        if !output.is_empty() {
            println!("{output}");
        }
        emit_human_warnings(&result.warnings);
        return Ok(());
    }

    let json = render_json(result)?;
    println!("{json}");
    Ok(())
}

fn render_json(result: CommandResult) -> Result<String, WavepeekError> {
    let envelope =
        OutputEnvelope::with_warnings(result.command.as_str(), result.data, result.warnings);
    serde_json::to_string(&envelope)
        .map_err(|error| WavepeekError::Internal(format!("failed to serialize output: {error}")))
}

fn render_human(data: &CommandData) -> String {
    match data {
        CommandData::Info(info) => {
            let mut lines = Vec::new();
            lines.push(format!("time_unit: {}", info.time_unit));
            lines.push(format!("time_precision: {}", info.time_precision));
            lines.push(format!("time_start: {}", info.time_start));
            lines.push(format!("time_end: {}", info.time_end));
            lines.join("\n")
        }
        CommandData::Tree(scopes) => scopes
            .iter()
            .map(|entry| format!("{} {}", entry.depth, entry.path))
            .collect::<Vec<_>>()
            .join("\n"),
        CommandData::Signals(signals) => signals
            .iter()
            .map(|entry| match entry.width {
                Some(width) => {
                    format!(
                        "{} {} kind={} width={width}",
                        entry.path, entry.name, entry.kind
                    )
                }
                None => format!("{} {} kind={}", entry.path, entry.name, entry.kind),
            })
            .collect::<Vec<_>>()
            .join("\n"),
    }
}

fn emit_human_warnings(warnings: &[String]) {
    for warning in warnings {
        eprintln!("warning: {warning}");
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use crate::engine::{CommandData, CommandName, CommandResult};

    use super::{SCHEMA_VERSION, render_json};

    #[test]
    fn json_envelope_has_required_shape_for_info() {
        let result = CommandResult {
            command: CommandName::Info,
            human: false,
            data: CommandData::Info(crate::engine::info::InfoData {
                time_unit: "1ns".to_string(),
                time_precision: "1ns".to_string(),
                time_start: "0ns".to_string(),
                time_end: "10ns".to_string(),
            }),
            warnings: vec![],
        };

        let json = render_json(result).expect("json serialization should succeed");
        let value: Value = serde_json::from_str(&json).expect("json should parse");

        assert_eq!(value["schema_version"], SCHEMA_VERSION);
        assert_eq!(value["command"], "info");
        assert!(value["data"].is_object());
        assert!(value["warnings"].is_array());
    }

    #[test]
    fn json_envelope_preserves_warnings_for_tree() {
        let result = CommandResult {
            command: CommandName::Tree,
            human: false,
            data: CommandData::Tree(vec![crate::engine::tree::TreeEntry {
                path: "top.cpu".to_string(),
                depth: 1,
            }]),
            warnings: vec!["truncated to 1 entries".to_string()],
        };

        let json = render_json(result).expect("json serialization should succeed");
        let value: Value = serde_json::from_str(&json).expect("json should parse");

        assert_eq!(value["command"], "tree");
        assert_eq!(value["warnings"][0], "truncated to 1 entries");
        assert_eq!(value["data"][0]["path"], "top.cpu");
        assert_eq!(value["data"][0]["depth"], 1);
    }
}
