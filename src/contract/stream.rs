use serde::Serialize;

use crate::diagnostic::Diagnostic;
use crate::engine::CommandName;
use crate::error::WavepeekError;

use super::common::ContractDiagnostic;
use super::output::{
    ChangeSnapshot, ExtractAhbContext, ExtractAhbEvent, ExtractApbContext, ExtractApbEvent,
    ExtractAtbContext, ExtractAtbEvent, ExtractAxiContext, ExtractAxiStreamContext,
    ExtractAxiStreamTransfer, ExtractAxiTransfer, ExtractGenericRow, InfoData, OutputContextData,
    PropertyRow, ScopeEntry, SignalEntry, ValueSnapshot,
};

#[derive(Debug, Serialize)]
pub struct BeginRecord<'a> {
    #[serde(rename = "type")]
    record_type: &'static str,
    seq: usize,
    command: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    context: Option<OutputContextData<'a>>,
}

impl BeginRecord<'static> {
    pub fn new(seq: usize, command: CommandName) -> Result<Self, WavepeekError> {
        require_stream_command(command)?;
        Ok(Self {
            record_type: "begin",
            seq,
            command: command.as_str(),
            context: None,
        })
    }
}

impl<'a> BeginRecord<'a> {
    pub fn with_context<T: StreamContext + ?Sized>(
        seq: usize,
        command: CommandName,
        context: &'a T,
    ) -> Result<Self, WavepeekError> {
        require_stream_command(command)?;
        Ok(Self {
            record_type: "begin",
            seq,
            command: command.as_str(),
            context: Some(context.stream_context(command)?),
        })
    }
}

#[derive(Debug, Serialize)]
pub struct DataRecord<'a> {
    #[serde(rename = "type")]
    record_type: &'static str,
    seq: usize,
    data: StreamData<'a>,
}

impl<'a> DataRecord<'a> {
    pub fn new<T: StreamDataRow + ?Sized>(
        seq: usize,
        command: CommandName,
        data: &'a T,
    ) -> Result<Self, WavepeekError> {
        require_stream_command(command)?;
        Ok(Self {
            record_type: "data",
            seq,
            data: data.stream_data(command)?,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct DiagnosticRecord<'a> {
    #[serde(rename = "type")]
    record_type: &'static str,
    seq: usize,
    diagnostic: ContractDiagnostic<'a>,
}

impl<'a> DiagnosticRecord<'a> {
    pub fn new(
        seq: usize,
        command: CommandName,
        diagnostic: &'a Diagnostic,
    ) -> Result<Self, WavepeekError> {
        require_stream_command(command)?;
        Ok(Self {
            record_type: "diagnostic",
            seq,
            diagnostic: ContractDiagnostic::from_diagnostic(diagnostic)?,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct EndRecord {
    #[serde(rename = "type")]
    record_type: &'static str,
    seq: usize,
    records: RecordCounts,
}

impl EndRecord {
    pub fn new(
        seq: usize,
        command: CommandName,
        data: usize,
        diagnostics: usize,
    ) -> Result<Self, WavepeekError> {
        require_stream_command(command)?;
        Ok(Self {
            record_type: "end",
            seq,
            records: RecordCounts { data, diagnostics },
        })
    }
}

#[derive(Debug, Serialize)]
struct RecordCounts {
    data: usize,
    diagnostics: usize,
}

pub trait StreamContext {
    fn stream_context(&self, command: CommandName) -> Result<OutputContextData<'_>, WavepeekError>;
}

macro_rules! impl_stream_context {
    ($source:ty, $command:expr, $variant:ident, $contract:ident) => {
        impl StreamContext for $source {
            fn stream_context(
                &self,
                command: CommandName,
            ) -> Result<OutputContextData<'_>, WavepeekError> {
                require_data_command(command, $command)?;
                Ok(OutputContextData::$variant($contract::from(self)))
            }
        }
    };
}

impl_stream_context!(
    crate::engine::ahb::AhbContext,
    CommandName::ExtractAhb,
    Ahb,
    ExtractAhbContext
);
impl_stream_context!(
    crate::engine::apb::ApbContext,
    CommandName::ExtractApb,
    Apb,
    ExtractApbContext
);
impl_stream_context!(
    crate::engine::atb::AtbContext,
    CommandName::ExtractAtb,
    Atb,
    ExtractAtbContext
);
impl_stream_context!(
    crate::engine::axi::AxiContext,
    CommandName::ExtractAxi,
    Axi,
    ExtractAxiContext
);
impl_stream_context!(
    crate::engine::axistream::AxiStreamContext,
    CommandName::ExtractAxiStream,
    AxiStream,
    ExtractAxiStreamContext
);

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum StreamData<'a> {
    Info(InfoData<'a>),
    Scope(ScopeEntry<'a>),
    Signal(SignalEntry<'a>),
    Value(ValueSnapshot<'a>),
    Change(ChangeSnapshot<'a>),
    Property(PropertyRow<'a>),
    ExtractAhb(ExtractAhbEvent<'a>),
    ExtractApb(ExtractApbEvent<'a>),
    ExtractAtb(ExtractAtbEvent<'a>),
    ExtractAxi(ExtractAxiTransfer<'a>),
    ExtractAxiStream(ExtractAxiStreamTransfer<'a>),
    ExtractGeneric(ExtractGenericRow<'a>),
}

pub trait StreamDataRow {
    fn stream_data(&self, command: CommandName) -> Result<StreamData<'_>, WavepeekError>;
}

impl StreamDataRow for crate::engine::info::InfoData {
    fn stream_data(&self, command: CommandName) -> Result<StreamData<'_>, WavepeekError> {
        require_data_command(command, CommandName::Info)?;
        Ok(StreamData::Info(InfoData::from(self)))
    }
}

impl StreamDataRow for crate::engine::scope::ScopeEntry {
    fn stream_data(&self, command: CommandName) -> Result<StreamData<'_>, WavepeekError> {
        require_data_command(command, CommandName::Scope)?;
        Ok(StreamData::Scope(ScopeEntry::try_from(self)?))
    }
}

impl StreamDataRow for crate::engine::signal::SignalEntry {
    fn stream_data(&self, command: CommandName) -> Result<StreamData<'_>, WavepeekError> {
        require_data_command(command, CommandName::Signal)?;
        Ok(StreamData::Signal(SignalEntry::try_from(self)?))
    }
}

impl StreamDataRow for crate::engine::value::ValueSnapshot {
    fn stream_data(&self, command: CommandName) -> Result<StreamData<'_>, WavepeekError> {
        require_data_command(command, CommandName::Value)?;
        Ok(StreamData::Value(ValueSnapshot::from(self)))
    }
}

impl StreamDataRow for crate::engine::change::ChangeSnapshot {
    fn stream_data(&self, command: CommandName) -> Result<StreamData<'_>, WavepeekError> {
        require_data_command(command, CommandName::Change)?;
        Ok(StreamData::Change(ChangeSnapshot::from(self)))
    }
}

impl StreamDataRow for crate::engine::property::PropertyCaptureRow {
    fn stream_data(&self, command: CommandName) -> Result<StreamData<'_>, WavepeekError> {
        require_data_command(command, CommandName::Property)?;
        Ok(StreamData::Property(PropertyRow::from(self)))
    }
}

impl StreamDataRow for crate::engine::ahb::AhbEvent {
    fn stream_data(&self, command: CommandName) -> Result<StreamData<'_>, WavepeekError> {
        require_data_command(command, CommandName::ExtractAhb)?;
        Ok(StreamData::ExtractAhb(ExtractAhbEvent::from(self)))
    }
}

impl StreamDataRow for crate::engine::apb::ApbEvent {
    fn stream_data(&self, command: CommandName) -> Result<StreamData<'_>, WavepeekError> {
        require_data_command(command, CommandName::ExtractApb)?;
        Ok(StreamData::ExtractApb(ExtractApbEvent::from(self)))
    }
}

impl StreamDataRow for crate::engine::atb::AtbEvent {
    fn stream_data(&self, command: CommandName) -> Result<StreamData<'_>, WavepeekError> {
        require_data_command(command, CommandName::ExtractAtb)?;
        Ok(StreamData::ExtractAtb(ExtractAtbEvent::from(self)))
    }
}

impl StreamDataRow for crate::engine::axi::AxiTransfer {
    fn stream_data(&self, command: CommandName) -> Result<StreamData<'_>, WavepeekError> {
        require_data_command(command, CommandName::ExtractAxi)?;
        Ok(StreamData::ExtractAxi(ExtractAxiTransfer::from(self)))
    }
}

impl StreamDataRow for crate::engine::axistream::AxiStreamTransfer {
    fn stream_data(&self, command: CommandName) -> Result<StreamData<'_>, WavepeekError> {
        require_data_command(command, CommandName::ExtractAxiStream)?;
        Ok(StreamData::ExtractAxiStream(
            ExtractAxiStreamTransfer::from(self),
        ))
    }
}

impl StreamDataRow for crate::engine::extract::ExtractGenericRow {
    fn stream_data(&self, command: CommandName) -> Result<StreamData<'_>, WavepeekError> {
        require_data_command(command, CommandName::ExtractGeneric)?;
        Ok(StreamData::ExtractGeneric(ExtractGenericRow::from(self)))
    }
}

fn require_data_command(actual: CommandName, expected: CommandName) -> Result<(), WavepeekError> {
    if actual == expected {
        Ok(())
    } else {
        Err(WavepeekError::Internal(format!(
            "JSONL data for {} cannot be written to {} stream",
            expected.as_str(),
            actual.as_str()
        )))
    }
}

fn require_stream_command(command: CommandName) -> Result<(), WavepeekError> {
    match command {
        CommandName::Info
        | CommandName::Scope
        | CommandName::Signal
        | CommandName::Value
        | CommandName::Change
        | CommandName::Property
        | CommandName::ExtractAhb
        | CommandName::ExtractApb
        | CommandName::ExtractAtb
        | CommandName::ExtractAxi
        | CommandName::ExtractAxiStream
        | CommandName::ExtractGeneric => Ok(()),
        _ => Err(WavepeekError::Args(
            "--jsonl is available only for waveform commands".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::engine::CommandName;

    use super::{BeginRecord, DataRecord};

    #[test]
    fn begin_record_has_stable_shape() {
        let value = serde_json::to_value(
            BeginRecord::new(0, CommandName::Change).expect("change begin should convert"),
        )
        .expect("begin record should serialize");

        assert_eq!(
            value,
            json!({"type": "begin", "seq": 0, "command": "change"})
        );
    }

    #[test]
    fn data_record_rejects_command_mismatch() {
        let item = crate::engine::info::InfoData {
            time_unit: "1ns".to_string(),
            time_start: "0ns".to_string(),
            time_end: "10ns".to_string(),
        };

        assert!(DataRecord::new(1, CommandName::Change, &item).is_err());
    }

    #[test]
    fn data_record_uses_contract_payload_shape() {
        let item = crate::engine::signal::SignalEntry {
            name: "clk".to_string(),
            path: "top.clk".to_string(),
            relative_path: "clk".to_string(),
            kind: "wire".to_string(),
            width: Some(1),
        };
        let value = serde_json::to_value(
            DataRecord::new(1, CommandName::Signal, &item).expect("signal data should convert"),
        )
        .expect("data record should serialize");

        assert_eq!(
            value,
            json!({
                "type": "data",
                "seq": 1,
                "data": {"name": "clk", "path": "top.clk", "relative_path": "clk", "kind": "wire", "width": 1}
            })
        );
    }
}
