use std::io::{self, Write};

use serde::Serialize;

use crate::contract::{output::OutputEnvelope, stream};
use crate::diagnostic::{Diagnostic, DiagnosticKind};
use crate::engine::{CommandData, CommandName, CommandResult, HumanRenderOptions};
use crate::error::WavepeekError;
use crate::output_mode::OutputMode;

pub struct JsonlWriter<W: Write> {
    writer: W,
    command: CommandName,
    next_seq: usize,
    data: usize,
    diagnostics: usize,
}

impl<W: Write> JsonlWriter<W> {
    pub const fn new(writer: W, command: CommandName) -> Self {
        Self {
            writer,
            command,
            next_seq: 0,
            data: 0,
            diagnostics: 0,
        }
    }

    pub fn begin(&mut self) -> Result<(), WavepeekError> {
        let record = stream::BeginRecord::new(self.next_seq, self.command)?;
        self.write_record(&record)
    }

    pub fn begin_context<T: stream::StreamContext + ?Sized>(
        &mut self,
        context: &T,
    ) -> Result<(), WavepeekError> {
        let record = stream::BeginRecord::with_context(self.next_seq, self.command, context)?;
        self.write_record(&record)
    }

    pub fn data<T: stream::StreamDataRow + ?Sized>(
        &mut self,
        data: &T,
    ) -> Result<(), WavepeekError> {
        let record = stream::DataRecord::new(self.next_seq, self.command, data)?;
        self.write_record(&record)?;
        self.data += 1;
        Ok(())
    }

    pub fn diagnostic(&mut self, diagnostic: &Diagnostic) -> Result<(), WavepeekError> {
        let record = stream::DiagnosticRecord::new(self.next_seq, self.command, diagnostic)?;
        self.write_record(&record)?;
        self.diagnostics += 1;
        Ok(())
    }

    pub fn end(&mut self) -> Result<(), WavepeekError> {
        let record =
            stream::EndRecord::new(self.next_seq, self.command, self.data, self.diagnostics)?;
        self.write_record(&record)
    }

    #[cfg(test)]
    pub const fn data_count(&self) -> usize {
        self.data
    }

    #[cfg(test)]
    pub const fn diagnostic_count(&self) -> usize {
        self.diagnostics
    }

    fn write_record<T: Serialize>(&mut self, record: &T) -> Result<(), WavepeekError> {
        serde_json::to_writer(&mut self.writer, record).map_err(map_jsonl_serde_error)?;
        self.writer.write_all(b"\n").map_err(map_stdout_io_error)?;
        self.writer.flush().map_err(map_stdout_io_error)?;
        self.next_seq += 1;
        Ok(())
    }
}

pub fn write(result: CommandResult) -> Result<(), WavepeekError> {
    match result.output_mode {
        OutputMode::Human => {
            let output = render_human(&result.data, result.human_options);
            if !output.is_empty() {
                write_stdout(output.as_str())?;
            }
            emit_human_diagnostics(&result.diagnostics);
            Ok(())
        }
        OutputMode::Json => {
            let json = render_json(result)?;
            write_stdout(&json)
        }
        OutputMode::Jsonl => {
            let stdout = io::stdout();
            let mut writer = JsonlWriter::new(stdout.lock(), result.command);
            write_jsonl_result(result, &mut writer)
        }
    }
}

pub fn write_jsonl_result<W: Write>(
    result: CommandResult,
    writer: &mut JsonlWriter<W>,
) -> Result<(), WavepeekError> {
    if !matches!(
        &result.data,
        CommandData::ExtractAhb(_)
            | CommandData::ExtractApb(_)
            | CommandData::ExtractAtb(_)
            | CommandData::ExtractAxi(_)
            | CommandData::ExtractAxiStream(_)
    ) {
        writer.begin()?;
    }
    match &result.data {
        CommandData::Info(data) => writer.data(data)?,
        CommandData::Scope(entries) => {
            for entry in entries {
                writer.data(entry)?;
            }
        }
        CommandData::Signal(entries) => {
            for entry in entries {
                writer.data(entry)?;
            }
        }
        CommandData::Value(snapshots) => {
            for snapshot in snapshots {
                writer.data(snapshot)?;
            }
        }
        CommandData::Change(snapshots) => {
            for snapshot in snapshots {
                writer.data(snapshot)?;
            }
        }
        CommandData::Property(rows) => {
            for row in rows {
                writer.data(row)?;
            }
        }
        CommandData::ExtractAhb(data) => {
            writer.begin_context(&data.context())?;
            for event in &data.events {
                writer.data(event)?;
            }
        }
        CommandData::ExtractApb(data) => {
            writer.begin_context(&data.context())?;
            for event in &data.events {
                writer.data(event)?;
            }
        }
        CommandData::ExtractAtb(data) => {
            writer.begin_context(&data.context())?;
            for event in &data.events {
                writer.data(event)?;
            }
        }
        CommandData::ExtractAxi(data) => {
            writer.begin_context(&data.context())?;
            for transfer in &data.transfers {
                writer.data(transfer)?;
            }
        }
        CommandData::ExtractAxiStream(data) => {
            writer.begin_context(&data.context())?;
            for transfer in &data.transfers {
                writer.data(transfer)?;
            }
        }
        CommandData::ExtractGeneric(data) => {
            for row in &data.rows {
                writer.data(row)?;
            }
        }
        CommandData::Text(_) => {
            return Err(WavepeekError::Args(
                "--jsonl is available only for waveform commands".to_string(),
            ));
        }
    }

    for diagnostic in &result.diagnostics {
        writer.diagnostic(diagnostic)?;
    }
    writer.end()
}

fn map_jsonl_serde_error(error: serde_json::Error) -> WavepeekError {
    if error.io_error_kind() == Some(io::ErrorKind::BrokenPipe) {
        WavepeekError::BrokenPipe
    } else {
        WavepeekError::Internal(format!("failed to serialize JSONL output: {error}"))
    }
}

pub(crate) fn map_stdout_io_error(error: io::Error) -> WavepeekError {
    if error.kind() == io::ErrorKind::BrokenPipe {
        WavepeekError::BrokenPipe
    } else {
        WavepeekError::Internal(format!("failed to write stdout: {error}"))
    }
}

fn render_json(result: CommandResult) -> Result<String, WavepeekError> {
    let envelope = OutputEnvelope::from_result(&result)?;
    serde_json::to_string(&envelope)
        .map_err(|error| WavepeekError::Internal(format!("failed to serialize output: {error}")))
}

fn render_human(data: &CommandData, options: HumanRenderOptions) -> String {
    match data {
        CommandData::Text(text) => text.clone(),
        CommandData::Info(info) => {
            let mut lines = Vec::new();
            lines.push(format!("time_unit: {}", info.time_unit));
            lines.push(format!("time_start: {}", info.time_start));
            lines.push(format!("time_end: {}", info.time_end));
            lines.join("\n")
        }
        CommandData::Scope(scopes) => {
            if options.scope_tree {
                render_scope_tree(scopes)
            } else {
                scopes
                    .iter()
                    .map(|entry| format!("{} {} kind={}", entry.depth, entry.path, entry.kind))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
        CommandData::Signal(signals) => signals
            .iter()
            .map(|entry| match entry.width {
                Some(width) => {
                    format!(
                        "{} kind={} width={width}",
                        signal_display_name(entry, options.signals_abs),
                        entry.kind
                    )
                }
                None => format!(
                    "{} kind={}",
                    signal_display_name(entry, options.signals_abs),
                    entry.kind
                ),
            })
            .collect::<Vec<_>>()
            .join("\n"),
        CommandData::Value(snapshots) => snapshots
            .iter()
            .map(|snapshot| {
                let mut parts = Vec::with_capacity(snapshot.signals.len() + 1);
                parts.push(format!("@{}", snapshot.time));
                for signal in &snapshot.signals {
                    let display = if options.signals_abs {
                        signal.path.as_str()
                    } else {
                        signal.display.as_str()
                    };
                    parts.push(format!("{display}={}", signal.value));
                }
                parts.join(" ")
            })
            .collect::<Vec<_>>()
            .join("\n"),
        CommandData::Change(snapshots) => snapshots
            .iter()
            .map(|snapshot| {
                let mut parts = Vec::with_capacity(snapshot.signals.len() + 2);
                parts.push(format!("@{}", snapshot.time));
                if snapshot.sample_time != snapshot.time {
                    parts.push(format!("sample@{}", snapshot.sample_time));
                }
                for signal in &snapshot.signals {
                    let display = if options.signals_abs {
                        signal.path.as_str()
                    } else {
                        signal.display.as_str()
                    };
                    parts.push(format!("{display}={}", signal.value));
                }
                parts.join(" ")
            })
            .collect::<Vec<_>>()
            .join("\n"),
        CommandData::Property(rows) => rows
            .iter()
            .map(|row| {
                if row.sample_time == row.time {
                    format!("@{} {}", row.time, row.kind)
                } else {
                    format!("@{} sample@{} {}", row.time, row.sample_time, row.kind)
                }
            })
            .collect::<Vec<_>>()
            .join("\n"),
        CommandData::ExtractAhb(data) => render_ahb_human(data, options),
        CommandData::ExtractApb(data) => render_apb_human(data, options),
        CommandData::ExtractAtb(data) => render_atb_human(data, options),
        CommandData::ExtractAxi(data) => render_axi_human(data, options),
        CommandData::ExtractAxiStream(data) => render_axistream_human(data, options),
        CommandData::ExtractGeneric(data) => data
            .rows
            .iter()
            .map(|row| {
                let mut parts = Vec::with_capacity(row.payload.len() + 3);
                parts.push(format!("@{}", row.time));
                parts.push(format!("sample@{}", row.sample_time));
                if data.source_count > 1 {
                    parts.push(format!("[{}]", row.source));
                }
                for payload in &row.payload {
                    let display = if options.signals_abs {
                        payload.path.as_str()
                    } else {
                        payload.display.as_str()
                    };
                    parts.push(format!("{display}={}", payload.value));
                }
                parts.join(" ")
            })
            .collect::<Vec<_>>()
            .join("\n"),
    }
}

fn render_ahb_human(data: &crate::engine::ahb::AhbData, options: HumanRenderOptions) -> String {
    let mut lines = Vec::new();
    lines.push(format!("name: {}", data.name));
    lines.push(format!("profile: {}", data.profile));
    lines.push(format!("issue: {}", data.issue));
    lines.push(format!("include_stall: {}", data.include_stall));
    lines.push(format!("include_idle: {}", data.include_idle));
    lines.push(format!("include_busy: {}", data.include_busy));
    lines.push(format!(
        "initial_data_phase: {}",
        data.initial_data_phase.state
    ));
    lines.push("mappings:".to_string());
    for mapping in &data.mappings {
        let display = if options.signals_abs {
            mapping.path.as_str()
        } else {
            mapping.display.as_str()
        };
        lines.push(format!("  {} = {display}", mapping.standard));
    }
    lines.push("events:".to_string());
    for event in &data.events {
        let mut label = vec![event.event.as_str()];
        if let Some(transfer) = event.transfer.as_deref()
            && transfer != event.event
        {
            label.push(transfer);
        }
        if let Some(direction) = event.direction.as_deref() {
            label.push(direction);
        }
        let mut parts = Vec::with_capacity(event.payload.len() + 3);
        parts.push(format!("@{}", event.time));
        parts.push(format!("sample@{}", event.sample_time));
        parts.push(format!("[{}]", label.join(" ")));
        for payload in &event.payload {
            let display = if options.signals_abs {
                payload.path.as_str()
            } else {
                payload.standard.as_str()
            };
            parts.push(format!("{display}={}", payload.value));
        }
        lines.push(parts.join(" "));
    }
    lines.join("\n")
}

fn render_apb_human(data: &crate::engine::apb::ApbData, options: HumanRenderOptions) -> String {
    let mut lines = Vec::new();
    lines.push(format!("name: {}", data.name));
    lines.push(format!("profile: {}", data.profile));
    lines.push(format!("issue: {}", data.issue));
    lines.push(format!("pready_mode: {}", data.pready_mode));
    lines.push(format!("include_wait: {}", data.include_wait));
    lines.push("mappings:".to_string());
    for mapping in &data.mappings {
        let display = if options.signals_abs {
            mapping.path.as_str()
        } else {
            mapping.display.as_str()
        };
        lines.push(format!("  {} = {display}", mapping.standard));
    }
    lines.push("events:".to_string());
    for event in &data.events {
        let mut parts = Vec::with_capacity(event.payload.len() + 3);
        parts.push(format!("@{}", event.time));
        parts.push(format!("sample@{}", event.sample_time));
        parts.push(format!("[{} {}]", event.event, event.direction));
        for payload in &event.payload {
            let display = if options.signals_abs {
                payload.path.as_str()
            } else {
                payload.standard.as_str()
            };
            parts.push(format!("{display}={}", payload.value));
        }
        lines.push(parts.join(" "));
    }
    lines.join("\n")
}

fn render_atb_human(data: &crate::engine::atb::AtbData, options: HumanRenderOptions) -> String {
    let mut lines = Vec::new();
    lines.push(format!("name: {}", data.name));
    lines.push(format!("profile: {}", data.profile));
    lines.push(format!("issue: {}", data.issue));
    lines.push("mappings:".to_string());
    for mapping in &data.mappings {
        let display = if options.signals_abs {
            mapping.path.as_str()
        } else {
            mapping.display.as_str()
        };
        lines.push(format!("  {} = {display}", mapping.standard));
    }
    lines.push("events:".to_string());
    for event in &data.events {
        let mut parts = Vec::with_capacity(event.payload.len() + 3);
        parts.push(format!("@{}", event.time));
        parts.push(format!("sample@{}", event.sample_time));
        parts.push(format!("[{}]", event.event.as_str()));
        for payload in &event.payload {
            let display = if options.signals_abs {
                payload.path.as_str()
            } else {
                payload.standard.as_str()
            };
            parts.push(format!("{display}={}", payload.value));
        }
        lines.push(parts.join(" "));
    }
    lines.join("\n")
}

fn render_axi_human(data: &crate::engine::axi::AxiData, options: HumanRenderOptions) -> String {
    let mut lines = Vec::new();
    lines.push(format!("name: {}", data.name));
    lines.push(format!("profile: {}", data.profile));
    lines.push(format!("issue: {}", data.issue));
    lines.push("mappings:".to_string());
    for mapping in &data.mappings {
        let display = if options.signals_abs {
            mapping.path.as_str()
        } else {
            mapping.display.as_str()
        };
        lines.push(format!("  {} = {display}", mapping.standard));
    }
    lines.push("transfers:".to_string());
    for transfer in &data.transfers {
        let mut parts = Vec::with_capacity(transfer.payload.len() + 3);
        parts.push(format!("@{}", transfer.time));
        parts.push(format!("sample@{}", transfer.sample_time));
        parts.push(format!("[{}]", transfer.channel));
        for payload in &transfer.payload {
            let display = if options.signals_abs {
                payload.path.as_str()
            } else {
                payload.standard.as_str()
            };
            parts.push(format!("{display}={}", payload.value));
        }
        lines.push(parts.join(" "));
    }
    lines.join("\n")
}

fn render_axistream_human(
    data: &crate::engine::axistream::AxiStreamData,
    options: HumanRenderOptions,
) -> String {
    let mut lines = Vec::new();
    lines.push(format!("name: {}", data.name));
    lines.push(format!("profile: {}", data.profile));
    lines.push(format!("issue: {}", data.issue));
    lines.push(format!("tready_mode: {}", data.tready_mode));
    lines.push("mappings:".to_string());
    for mapping in &data.mappings {
        let display = if options.signals_abs {
            mapping.path.as_str()
        } else {
            mapping.display.as_str()
        };
        lines.push(format!("  {} = {display}", mapping.standard));
    }
    lines.push("transfers:".to_string());
    for transfer in &data.transfers {
        let mut parts = Vec::with_capacity(transfer.payload.len() + 2);
        parts.push(format!("@{}", transfer.time));
        parts.push(format!("sample@{}", transfer.sample_time));
        for payload in &transfer.payload {
            let display = if options.signals_abs {
                payload.path.as_str()
            } else {
                payload.standard.as_str()
            };
            parts.push(format!("{display}={}", payload.value));
        }
        lines.push(parts.join(" "));
    }
    lines.join("\n")
}

fn render_scope_tree(scopes: &[crate::engine::scope::ScopeEntry]) -> String {
    if scopes.is_empty() {
        return String::new();
    }

    let mut lines = Vec::with_capacity(scopes.len());
    let mut ancestor_last = Vec::new();

    for (index, entry) in scopes.iter().enumerate() {
        let label = entry.path.rsplit('.').next().unwrap_or(entry.path.as_str());
        let scope_label = format!("{label} kind={}", entry.kind);
        let is_last = scope_entry_is_last_sibling(scopes, index);

        if entry.depth == 0 {
            lines.push(scope_label);
        } else {
            let mut line = String::new();

            for depth in 1..entry.depth {
                let ancestor_is_last = ancestor_last.get(depth).copied().unwrap_or(true);
                if ancestor_is_last {
                    line.push_str("    ");
                } else {
                    line.push_str("│   ");
                }
            }

            line.push_str(if is_last { "└── " } else { "├── " });
            line.push_str(scope_label.as_str());
            lines.push(line);
        }

        ancestor_last.truncate(entry.depth);
        ancestor_last.push(is_last);
    }

    lines.join("\n")
}

fn scope_entry_is_last_sibling(scopes: &[crate::engine::scope::ScopeEntry], index: usize) -> bool {
    let depth = scopes[index].depth;
    for next in scopes.iter().skip(index + 1) {
        if next.depth < depth {
            return true;
        }
        if next.depth == depth {
            return false;
        }
    }

    true
}

fn signal_display_name(entry: &crate::engine::signal::SignalEntry, abs: bool) -> &str {
    if abs {
        entry.path.as_str()
    } else {
        entry.display.as_str()
    }
}

fn emit_human_diagnostics(diagnostics: &[Diagnostic]) {
    for diagnostic in diagnostics {
        match diagnostic.kind() {
            DiagnosticKind::Info => eprintln!("info: {}", diagnostic.message()),
            DiagnosticKind::Warning => eprintln!(
                "warning[{}]: {}",
                diagnostic
                    .code()
                    .expect("warning diagnostics must have stable codes"),
                diagnostic.message()
            ),
            DiagnosticKind::Error => eprintln!(
                "error[{}]: {}",
                diagnostic
                    .code()
                    .expect("error diagnostics must have stable codes"),
                diagnostic.message()
            ),
        }
    }
}

pub(crate) fn write_stdout(output: &str) -> Result<(), WavepeekError> {
    let stdout = io::stdout();
    let mut writer = stdout.lock();
    writeln!(writer, "{}", output.strip_suffix('\n').unwrap_or(output)).map_err(map_stdout_io_error)
}

#[cfg(test)]
#[path = "tests/output_rendering_edges.rs"]
mod output_rendering_edges;

#[cfg(test)]
mod tests {
    use std::io;

    use serde_json::Value;

    use crate::diagnostic::{Diagnostic, WarningDiagnosticCode};
    use crate::engine::{CommandData, CommandName, CommandResult, HumanRenderOptions};
    use crate::output_mode::OutputMode;

    use super::{
        JsonlWriter, render_human, render_json, render_scope_tree, scope_entry_is_last_sibling,
        signal_display_name, write, write_jsonl_result,
    };

    #[test]
    fn json_envelope_has_required_shape_for_info() {
        let result = CommandResult {
            command: CommandName::Info,
            output_mode: OutputMode::Json,
            human_options: HumanRenderOptions::default(),
            data: CommandData::Info(crate::engine::info::InfoData {
                time_unit: "1ns".to_string(),
                time_start: "0ns".to_string(),
                time_end: "10ns".to_string(),
            }),
            diagnostics: vec![],
        };

        let json = render_json(result).expect("json serialization should succeed");
        let value: Value = serde_json::from_str(&json).expect("json should parse");

        assert_eq!(value["type"], "result");
        assert_eq!(value["command"], "info");
        assert_eq!(value["data"].as_array().map(Vec::len), Some(1));
        assert_eq!(value["data"][0]["time_unit"], "1ns");
        assert!(value["diagnostics"].is_array());
        assert!(value.get("warnings").is_none());
    }

    #[test]
    fn json_envelope_preserves_diagnostics_for_scope() {
        let result = CommandResult {
            command: CommandName::Scope,
            output_mode: OutputMode::Json,
            human_options: HumanRenderOptions::default(),
            data: CommandData::Scope(vec![crate::engine::scope::ScopeEntry {
                path: "top.cpu".to_string(),
                depth: 1,
                kind: "module".to_string(),
            }]),
            diagnostics: vec![Diagnostic::warning(
                WarningDiagnosticCode::OutputTruncated,
                "truncated to 1 entries",
            )],
        };

        let json = render_json(result).expect("json serialization should succeed");
        let value: Value = serde_json::from_str(&json).expect("json should parse");

        assert_eq!(value["command"], "scope");
        assert_eq!(value["diagnostics"][0]["kind"], "warning");
        assert_eq!(value["diagnostics"][0]["code"], "WPK-W0002");
        assert_eq!(value["diagnostics"][0]["message"], "truncated to 1 entries");
        assert_eq!(value["data"][0]["path"], "top.cpu");
        assert_eq!(value["data"][0]["depth"], 1);
        assert_eq!(value["data"][0]["kind"], "module");
    }

    #[derive(Default)]
    struct FlushCountingSink {
        bytes: Vec<u8>,
        flushes: usize,
    }

    impl io::Write for FlushCountingSink {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.bytes.extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            self.flushes += 1;
            Ok(())
        }
    }

    struct BrokenPipeSink;

    impl io::Write for BrokenPipeSink {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Err(io::Error::from(io::ErrorKind::BrokenPipe))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn jsonl_writer_emits_ordered_records_and_flushes_each_line() {
        let mut sink = FlushCountingSink::default();
        {
            let mut writer = JsonlWriter::new(&mut sink, CommandName::Change);
            writer.begin().expect("begin record should write");
            writer
                .data(&crate::engine::change::ChangeSnapshot {
                    time: "5ns".to_string(),
                    sample_time: "5ns".to_string(),
                    signals: Vec::new(),
                })
                .expect("data record should write");
            writer
                .diagnostic(&Diagnostic::warning(
                    WarningDiagnosticCode::OutputTruncated,
                    "truncated output to 1 entries",
                ))
                .expect("diagnostic record should write");
            writer
                .data(&crate::engine::change::ChangeSnapshot {
                    time: "10ns".to_string(),
                    sample_time: "10ns".to_string(),
                    signals: Vec::new(),
                })
                .expect("interleaved data record should write");
            writer.end().expect("end record should write");
            assert_eq!(writer.data_count(), 2);
            assert_eq!(writer.diagnostic_count(), 1);
        }

        assert_eq!(sink.flushes, 5);
        let output = String::from_utf8(sink.bytes).expect("JSONL should be UTF-8");
        assert!(output.ends_with('\n'));
        let lines = output.lines().collect::<Vec<_>>();
        assert_eq!(lines.len(), 5);
        let records = lines
            .iter()
            .map(|line| serde_json::from_str::<Value>(line).expect("line should parse"))
            .collect::<Vec<_>>();

        assert_eq!(records[0]["type"], "begin");
        assert_eq!(records[0]["seq"], 0);
        assert_eq!(records[0]["command"], "change");
        assert_eq!(records[1]["type"], "data");
        assert_eq!(records[1]["seq"], 1);
        assert!(records[1].get("command").is_none());
        assert_eq!(records[2]["type"], "diagnostic");
        assert!(records[2].get("command").is_none());
        assert_eq!(records[2]["diagnostic"]["code"], "WPK-W0002");
        assert_eq!(records[3]["type"], "data");
        assert_eq!(records[3]["data"]["time"], "10ns");
        assert_eq!(records[4]["type"], "end");
        assert!(records[4].get("command").is_none());
        assert_eq!(records[4]["records"]["data"], 2);
        assert_eq!(records[4]["records"]["diagnostics"], 1);
    }

    #[test]
    fn jsonl_writer_maps_broken_pipe_without_internal_fatal() {
        let mut writer = JsonlWriter::new(BrokenPipeSink, CommandName::Info);
        let error = writer.begin().expect_err("broken pipe should be returned");
        assert!(matches!(error, crate::error::WavepeekError::BrokenPipe));
    }

    #[test]
    fn jsonl_result_adapter_emits_data_diagnostics_and_counts() {
        let result = CommandResult {
            command: CommandName::Scope,
            output_mode: OutputMode::Jsonl,
            human_options: HumanRenderOptions::default(),
            data: CommandData::Scope(vec![crate::engine::scope::ScopeEntry {
                path: "top".to_string(),
                depth: 0,
                kind: "module".to_string(),
            }]),
            diagnostics: vec![Diagnostic::warning(
                WarningDiagnosticCode::OutputTruncated,
                "truncated output to 1 entries",
            )],
        };
        let mut sink = Vec::new();
        let mut writer = JsonlWriter::new(&mut sink, CommandName::Scope);
        write_jsonl_result(result, &mut writer).expect("JSONL adapter should write");

        let output = String::from_utf8(sink).expect("JSONL should be UTF-8");
        let records = output
            .lines()
            .map(|line| serde_json::from_str::<Value>(line).expect("line should parse"))
            .collect::<Vec<_>>();
        assert_eq!(records[0]["type"], "begin");
        assert_eq!(records[1]["data"]["path"], "top");
        assert_eq!(records[2]["diagnostic"]["code"], "WPK-W0002");
        assert_eq!(records[3]["records"]["data"], 1);
        assert_eq!(records[3]["records"]["diagnostics"], 1);
    }

    #[test]
    fn property_rows_render_as_time_and_kind_lines() {
        let rendered = render_human(
            &CommandData::Property(vec![
                crate::engine::property::PropertyCaptureRow {
                    time: "10ns".to_string(),
                    sample_time: "10ns".to_string(),
                    kind: crate::engine::property::PropertyResultKind::Assert,
                },
                crate::engine::property::PropertyCaptureRow {
                    time: "25ns".to_string(),
                    sample_time: "24ns".to_string(),
                    kind: crate::engine::property::PropertyResultKind::Deassert,
                },
            ]),
            HumanRenderOptions::default(),
        );

        assert_eq!(rendered, "@10ns assert\n@25ns sample@24ns deassert");
    }

    #[test]
    fn scope_tree_render_matches_linux_tree_style() {
        let rendered = render_human(
            &CommandData::Scope(vec![
                crate::engine::scope::ScopeEntry {
                    path: "top".to_string(),
                    depth: 0,
                    kind: "module".to_string(),
                },
                crate::engine::scope::ScopeEntry {
                    path: "top.cpu".to_string(),
                    depth: 1,
                    kind: "module".to_string(),
                },
                crate::engine::scope::ScopeEntry {
                    path: "top.cpu.alu".to_string(),
                    depth: 2,
                    kind: "function".to_string(),
                },
                crate::engine::scope::ScopeEntry {
                    path: "top.cpu.regs".to_string(),
                    depth: 2,
                    kind: "module".to_string(),
                },
                crate::engine::scope::ScopeEntry {
                    path: "top.mem".to_string(),
                    depth: 1,
                    kind: "module".to_string(),
                },
            ]),
            HumanRenderOptions {
                scope_tree: true,
                signals_abs: false,
            },
        );

        assert_eq!(
            rendered,
            "top kind=module\n├── cpu kind=module\n│   ├── alu kind=function\n│   └── regs kind=module\n└── mem kind=module"
        );
    }

    #[test]
    fn value_human_render_is_deterministic_and_compact() {
        let rendered = render_human(
            &CommandData::Value(vec![crate::engine::value::ValueSnapshot {
                time: "10ns".to_string(),
                signals: vec![
                    crate::engine::value::ValueSignalValue {
                        display: "clk".to_string(),
                        path: "top.clk".to_string(),
                        value: "1'h1".to_string(),
                    },
                    crate::engine::value::ValueSignalValue {
                        display: "data".to_string(),
                        path: "top.data".to_string(),
                        value: "8'h0f".to_string(),
                    },
                ],
            }]),
            HumanRenderOptions::default(),
        );

        assert_eq!(rendered, "@10ns clk=1'h1 data=8'h0f");
    }

    #[test]
    fn change_human_render_is_single_line_per_snapshot() {
        let rendered = render_human(
            &CommandData::Change(vec![crate::engine::change::ChangeSnapshot {
                time: "5ns".to_string(),
                sample_time: "4ns".to_string(),
                signals: vec![
                    crate::engine::change::ChangeSignalValue {
                        display: "clk".to_string(),
                        path: "top.clk".to_string(),
                        value: "1'h1".to_string(),
                    },
                    crate::engine::change::ChangeSignalValue {
                        display: "data".to_string(),
                        path: "top.data".to_string(),
                        value: "8'h00".to_string(),
                    },
                ],
            }]),
            HumanRenderOptions::default(),
        );

        assert_eq!(rendered, "@5ns sample@4ns clk=1'h1 data=8'h00");
    }

    #[test]
    fn render_human_exercises_signal_variants() {
        let info = render_human(
            &CommandData::Info(crate::engine::info::InfoData {
                time_unit: "1ps".to_string(),
                time_start: "0ps".to_string(),
                time_end: "10ps".to_string(),
            }),
            HumanRenderOptions::default(),
        );
        assert_eq!(info, "time_unit: 1ps\ntime_start: 0ps\ntime_end: 10ps");

        let flat_scopes = render_human(
            &CommandData::Scope(vec![crate::engine::scope::ScopeEntry {
                path: "top.cpu".to_string(),
                depth: 1,
                kind: "module".to_string(),
            }]),
            HumanRenderOptions::default(),
        );
        assert_eq!(flat_scopes, "1 top.cpu kind=module");

        let signals = vec![
            crate::engine::signal::SignalEntry {
                display: "clk".to_string(),
                name: "clk".to_string(),
                path: "top.clk".to_string(),
                kind: "wire".to_string(),
                width: Some(1),
            },
            crate::engine::signal::SignalEntry {
                display: "status".to_string(),
                name: "status".to_string(),
                path: "top.status".to_string(),
                kind: "event".to_string(),
                width: None,
            },
        ];
        let rendered = render_human(
            &CommandData::Signal(signals.clone()),
            HumanRenderOptions {
                scope_tree: false,
                signals_abs: true,
            },
        );
        assert_eq!(rendered, "top.clk kind=wire width=1\ntop.status kind=event");
        assert_eq!(signal_display_name(&signals[0], true), "top.clk");
        assert_eq!(signal_display_name(&signals[0], false), "clk");
    }

    #[test]
    fn helper_renderers_exercise_empty_tree_and_sibling_detection() {
        assert_eq!(render_scope_tree(&[]), "");

        let scopes = vec![
            crate::engine::scope::ScopeEntry {
                path: "top".to_string(),
                depth: 0,
                kind: "module".to_string(),
            },
            crate::engine::scope::ScopeEntry {
                path: "top.cpu".to_string(),
                depth: 1,
                kind: "module".to_string(),
            },
            crate::engine::scope::ScopeEntry {
                path: "top.mem".to_string(),
                depth: 1,
                kind: "module".to_string(),
            },
        ];
        assert!(!scope_entry_is_last_sibling(&scopes, 1));
        assert!(scope_entry_is_last_sibling(&scopes, 2));
    }

    #[test]
    fn write_entrypoint_preserves_existing_newline() {
        write(CommandResult {
            command: CommandName::Info,
            output_mode: OutputMode::Human,
            human_options: HumanRenderOptions::default(),
            data: CommandData::Text("already-newline\n".to_string()),
            diagnostics: Vec::new(),
        })
        .expect("newline-terminated human output should not add a second newline");
    }
}
