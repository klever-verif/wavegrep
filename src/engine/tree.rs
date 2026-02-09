use crate::cli::tree::TreeArgs;
use crate::engine::{CommandData, CommandName, CommandResult};
use crate::error::WavepeekError;
use crate::waveform::Waveform;
use regex::Regex;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TreeEntry {
    pub path: String,
    pub depth: usize,
}

pub fn run(args: TreeArgs) -> Result<CommandResult, WavepeekError> {
    if args.max == 0 {
        return Err(WavepeekError::Args(
            "--max must be greater than 0".to_string(),
        ));
    }

    let filter = Regex::new(args.filter.as_str()).map_err(|error| {
        WavepeekError::Args(format!("invalid regex '{}': {error}", args.filter))
    })?;

    let waveform = Waveform::open(args.waves.as_path())?;
    let mut entries = waveform
        .scopes_depth_first(args.max_depth)
        .into_iter()
        .filter(|entry| filter.is_match(entry.path.as_str()))
        .map(|entry| TreeEntry {
            path: entry.path,
            depth: entry.depth,
        })
        .collect::<Vec<_>>();

    let mut warnings = Vec::new();
    if entries.len() > args.max {
        entries.truncate(args.max);
        warnings.push(format!(
            "truncated output to {} entries (use --max to increase limit)",
            args.max
        ));
    }

    Ok(CommandResult {
        command: CommandName::Tree,
        human: args.human,
        data: CommandData::Tree(entries),
        warnings,
    })
}
