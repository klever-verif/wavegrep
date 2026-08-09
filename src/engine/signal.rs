use crate::cli::limits::LimitArg;
use crate::cli::signal::SignalArgs;
use crate::debug_trace::DebugTrace;
use crate::diagnostic::{Diagnostic, WarningDiagnosticCode};
use crate::engine::{CommandData, CommandName, CommandResult};
use crate::error::WavepeekError;
use crate::waveform::Waveform;
use regex::Regex;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SignalEntry {
    #[serde(skip_serializing)]
    pub display: String,
    pub name: String,
    pub path: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
}

pub fn run(args: SignalArgs) -> Result<CommandResult, WavepeekError> {
    let SignalArgs {
        waves,
        scope,
        max,
        filter,
        recursive,
        max_depth,
        abs,
        json,
        jsonl,
    } = args;

    if max == LimitArg::Numeric(0) {
        return Err(WavepeekError::Args(
            "--max must be greater than 0. See 'wavepeek signal --help'.".to_string(),
        ));
    }

    let filter = Regex::new(filter.as_str()).map_err(|error| {
        WavepeekError::Args(format!(
            "invalid regex '{}': {error}. See 'wavepeek signal --help'.",
            filter
        ))
    })?;

    let mut diagnostics = Vec::new();
    if max.is_unlimited() {
        diagnostics.push(Diagnostic::warning(
            WarningDiagnosticCode::LimitDisabled,
            "limit disabled: --max=unlimited",
        ));
    }
    if max_depth == LimitArg::Unlimited {
        diagnostics.push(Diagnostic::warning(
            WarningDiagnosticCode::LimitDisabled,
            "limit disabled: --max-depth=unlimited",
        ));
    }

    let effective_max_depth = match max_depth {
        LimitArg::Numeric(value) => Some(value),
        LimitArg::Unlimited => None,
    };
    let scope_prefix = format!("{scope}.");

    let debug = DebugTrace::for_command(CommandName::Signal);
    debug.event("backend.open.start", || serde_json::json!({}));
    let waveform = Waveform::open(waves.as_path())?;
    debug.event("backend.open.done", || {
        serde_json::json!({
            "backend": waveform.backend_name(),
            "format": waveform.format_name(),
        })
    });
    let listing = if recursive {
        waveform.signals_in_scope_recursive_report(scope.as_str(), effective_max_depth)?
    } else {
        waveform.signals_in_scope_report(scope.as_str())?
    };
    debug.event("signal.list.done", || {
        serde_json::json!({
            "signals": listing.entries.len(),
            "ambiguous_signals_omitted": listing.omitted_ambiguous_paths.len(),
        })
    });
    if !listing.omitted_ambiguous_paths.is_empty() {
        diagnostics.push(ambiguous_signal_warning(
            listing.omitted_ambiguous_paths.as_slice(),
        ));
    }
    let mut entries = listing
        .entries
        .into_iter()
        .filter(|entry| filter.is_match(entry.name.as_str()))
        .map(|entry| SignalEntry {
            display: signal_display_name(
                recursive,
                scope_prefix.as_str(),
                entry.path.as_str(),
                entry.name.as_str(),
            ),
            name: entry.name,
            path: entry.path,
            kind: entry.kind,
            width: entry.width,
        })
        .collect::<Vec<_>>();
    debug.event(
        "signal.filter.done",
        || serde_json::json!({"signals": entries.len()}),
    );

    if let Some(max_entries) = max.numeric()
        && entries.len() > max_entries
    {
        entries.truncate(max_entries);
        diagnostics.push(Diagnostic::warning(
            WarningDiagnosticCode::OutputTruncated,
            format!("truncated output to {max_entries} entries (use --max to increase limit)"),
        ));
    }

    if entries.is_empty() {
        diagnostics.push(Diagnostic::warning(
            WarningDiagnosticCode::EmptyResult,
            "no signals found in selected scope",
        ));
    }

    Ok(CommandResult {
        command: CommandName::Signal,
        output_mode: crate::output_mode::OutputMode::from_json_flags(json, jsonl),
        human_options: crate::engine::HumanRenderOptions {
            scope_tree: false,
            signals_abs: abs,
        },
        data: CommandData::Signal(entries),
        diagnostics,
    })
}

fn signal_display_name(recursive: bool, scope_prefix: &str, path: &str, name: &str) -> String {
    if !recursive {
        return name.to_string();
    }

    path.strip_prefix(scope_prefix).unwrap_or(name).to_string()
}

fn ambiguous_signal_warning(paths: &[String]) -> Diagnostic {
    const DISPLAY_LIMIT: usize = 5;

    let mut displayed = paths
        .iter()
        .take(DISPLAY_LIMIT)
        .map(|path| format!("'{path}'"))
        .collect::<Vec<_>>();
    if paths.len() > DISPLAY_LIMIT {
        displayed.push(format!("and {} more", paths.len() - DISPLAY_LIMIT));
    }
    let plural = if paths.len() == 1 { "" } else { "s" };
    Diagnostic::warning(
        WarningDiagnosticCode::AmbiguousSignalsOmitted,
        format!(
            "omitted {} ambiguous FSDB signal path{plural}: {}; no candidate was selected",
            paths.len(),
            displayed.join(", ")
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ambiguous_signal_warning_is_stable_and_bounded() {
        let diagnostic = ambiguous_signal_warning(&["top.opcode".to_string()]);
        assert_eq!(diagnostic.code(), Some("WPK-W0005"));
        assert_eq!(
            diagnostic.message(),
            "omitted 1 ambiguous FSDB signal path: 'top.opcode'; no candidate was selected"
        );

        let paths = (0..7)
            .map(|index| format!("top.signal{index}"))
            .collect::<Vec<_>>();
        assert_eq!(
            ambiguous_signal_warning(paths.as_slice()).message(),
            "omitted 7 ambiguous FSDB signal paths: 'top.signal0', 'top.signal1', 'top.signal2', 'top.signal3', 'top.signal4', and 2 more; no candidate was selected"
        );
    }
}
