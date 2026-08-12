use std::collections::BTreeMap;

use serde::Serialize;

use crate::diagnostic::Diagnostic;
use crate::engine::{CommandData, CommandName, CommandResult};
use crate::error::WavepeekError;

use super::common::{
    CanonicalPath, ContractDiagnostic, NormalizedTime, SampledValue, ScopeKind, SignalKind,
    validate_scope_kind, validate_signal_kind,
};

#[derive(Debug, Serialize)]
pub struct OutputEnvelope<'a> {
    command: &'static str,
    data: OutputData<'a>,
    diagnostics: Vec<ContractDiagnostic<'a>>,
}

impl<'a> OutputEnvelope<'a> {
    pub fn from_result(result: &'a CommandResult) -> Result<Self, WavepeekError> {
        Ok(Self {
            command: result.command.as_str(),
            data: OutputData::from_command_data(result.command, &result.data)?,
            diagnostics: diagnostics(&result.diagnostics)?,
        })
    }
}

fn diagnostics(diagnostics: &[Diagnostic]) -> Result<Vec<ContractDiagnostic<'_>>, WavepeekError> {
    diagnostics
        .iter()
        .map(ContractDiagnostic::from_diagnostic)
        .collect()
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum OutputData<'a> {
    Info(InfoData<'a>),
    Scope(Vec<ScopeEntry<'a>>),
    Signal(Vec<SignalEntry<'a>>),
    Value(Vec<ValueSnapshot<'a>>),
    Change(Vec<ChangeSnapshot<'a>>),
    Property(Vec<PropertyRow<'a>>),
    ExtractAhb(ExtractAhbData<'a>),
    ExtractApb(ExtractApbData<'a>),
    ExtractAtb(ExtractAtbData<'a>),
    ExtractAxi(ExtractAxiData<'a>),
    ExtractAxiStream(ExtractAxiStreamData<'a>),
    ExtractGeneric(Vec<ExtractGenericRow<'a>>),
    DocsTopics(DocsTopicsData<'a>),
    DocsSearch(DocsSearchData<'a>),
}

impl<'a> OutputData<'a> {
    pub fn from_command_data(
        command: CommandName,
        data: &'a CommandData,
    ) -> Result<Self, WavepeekError> {
        match (command, data) {
            (CommandName::Info, CommandData::Info(data)) => Ok(Self::Info(InfoData::from(data))),
            (CommandName::Scope, CommandData::Scope(entries)) => entries
                .iter()
                .map(ScopeEntry::try_from)
                .collect::<Result<Vec<_>, _>>()
                .map(Self::Scope),
            (CommandName::Signal, CommandData::Signal(entries)) => entries
                .iter()
                .map(SignalEntry::try_from)
                .collect::<Result<Vec<_>, _>>()
                .map(Self::Signal),
            (CommandName::Value, CommandData::Value(snapshots)) => Ok(Self::Value(
                snapshots.iter().map(ValueSnapshot::from).collect(),
            )),
            (CommandName::Change, CommandData::Change(snapshots)) => Ok(Self::Change(
                snapshots.iter().map(ChangeSnapshot::from).collect(),
            )),
            (CommandName::Property, CommandData::Property(rows)) => {
                Ok(Self::Property(rows.iter().map(PropertyRow::from).collect()))
            }
            (CommandName::ExtractAhb, CommandData::ExtractAhb(data)) => {
                Ok(Self::ExtractAhb(ExtractAhbData::from(data)))
            }
            (CommandName::ExtractApb, CommandData::ExtractApb(data)) => {
                Ok(Self::ExtractApb(ExtractApbData::from(data)))
            }
            (CommandName::ExtractAtb, CommandData::ExtractAtb(data)) => {
                Ok(Self::ExtractAtb(ExtractAtbData::from(data)))
            }
            (CommandName::ExtractAxi, CommandData::ExtractAxi(data)) => {
                Ok(Self::ExtractAxi(ExtractAxiData::from(data)))
            }
            (CommandName::ExtractAxiStream, CommandData::ExtractAxiStream(data)) => {
                Ok(Self::ExtractAxiStream(ExtractAxiStreamData::from(data)))
            }
            (CommandName::ExtractGeneric, CommandData::ExtractGeneric(data)) => Ok(
                Self::ExtractGeneric(data.rows.iter().map(ExtractGenericRow::from).collect()),
            ),
            (CommandName::DocsTopics, CommandData::DocsTopics(data)) => {
                Ok(Self::DocsTopics(DocsTopicsData::from(data)))
            }
            (CommandName::DocsSearch, CommandData::DocsSearch(data)) => {
                Ok(Self::DocsSearch(DocsSearchData::from(data)))
            }
            _ => Err(WavepeekError::Internal(format!(
                "command {} cannot be serialized as a JSON contract envelope",
                command.as_str()
            ))),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct InfoData<'a> {
    time_unit: &'a str,
    time_start: NormalizedTime<'a>,
    time_end: NormalizedTime<'a>,
}

impl<'a> From<&'a crate::engine::info::InfoData> for InfoData<'a> {
    fn from(data: &'a crate::engine::info::InfoData) -> Self {
        Self {
            time_unit: data.time_unit.as_str(),
            time_start: NormalizedTime::new(data.time_start.as_str()),
            time_end: NormalizedTime::new(data.time_end.as_str()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ScopeEntry<'a> {
    path: CanonicalPath<'a>,
    depth: usize,
    kind: ScopeKind<'a>,
}

impl<'a> TryFrom<&'a crate::engine::scope::ScopeEntry> for ScopeEntry<'a> {
    type Error = WavepeekError;

    fn try_from(entry: &'a crate::engine::scope::ScopeEntry) -> Result<Self, Self::Error> {
        Ok(Self {
            path: CanonicalPath::new(entry.path.as_str()),
            depth: entry.depth,
            kind: validate_scope_kind(entry.kind.as_str())?,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct SignalEntry<'a> {
    name: &'a str,
    path: CanonicalPath<'a>,
    kind: SignalKind<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    width: Option<u32>,
}

impl<'a> TryFrom<&'a crate::engine::signal::SignalEntry> for SignalEntry<'a> {
    type Error = WavepeekError;

    fn try_from(entry: &'a crate::engine::signal::SignalEntry) -> Result<Self, Self::Error> {
        Ok(Self {
            name: entry.name.as_str(),
            path: CanonicalPath::new(entry.path.as_str()),
            kind: validate_signal_kind(entry.kind.as_str())?,
            width: entry.width,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct SampledSignalValue<'a> {
    path: CanonicalPath<'a>,
    value: SampledValue<'a>,
}

impl<'a> From<&'a crate::engine::value::ValueSignalValue> for SampledSignalValue<'a> {
    fn from(signal: &'a crate::engine::value::ValueSignalValue) -> Self {
        Self {
            path: CanonicalPath::new(signal.path.as_str()),
            value: SampledValue::new(signal.value.as_str()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ChangeSignalValue<'a> {
    path: CanonicalPath<'a>,
    value: SampledValue<'a>,
}

impl<'a> From<&'a crate::engine::change::ChangeSignalValue> for ChangeSignalValue<'a> {
    fn from(signal: &'a crate::engine::change::ChangeSignalValue) -> Self {
        Self {
            path: CanonicalPath::new(signal.path.as_str()),
            value: SampledValue::new(signal.value.as_str()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ValueSnapshot<'a> {
    time: NormalizedTime<'a>,
    signals: Vec<SampledSignalValue<'a>>,
}

impl<'a> From<&'a crate::engine::value::ValueSnapshot> for ValueSnapshot<'a> {
    fn from(snapshot: &'a crate::engine::value::ValueSnapshot) -> Self {
        Self {
            time: NormalizedTime::new(snapshot.time.as_str()),
            signals: snapshot
                .signals
                .iter()
                .map(SampledSignalValue::from)
                .collect(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ChangeSnapshot<'a> {
    time: NormalizedTime<'a>,
    sample_time: NormalizedTime<'a>,
    signals: Vec<ChangeSignalValue<'a>>,
}

impl<'a> From<&'a crate::engine::change::ChangeSnapshot> for ChangeSnapshot<'a> {
    fn from(snapshot: &'a crate::engine::change::ChangeSnapshot) -> Self {
        Self {
            time: NormalizedTime::new(snapshot.time.as_str()),
            sample_time: NormalizedTime::new(snapshot.sample_time.as_str()),
            signals: snapshot
                .signals
                .iter()
                .map(ChangeSignalValue::from)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PropertyKind {
    Match,
    Assert,
    Deassert,
}

impl From<crate::engine::property::PropertyResultKind> for PropertyKind {
    fn from(kind: crate::engine::property::PropertyResultKind) -> Self {
        match kind {
            crate::engine::property::PropertyResultKind::Match => Self::Match,
            crate::engine::property::PropertyResultKind::Assert => Self::Assert,
            crate::engine::property::PropertyResultKind::Deassert => Self::Deassert,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PropertyRow<'a> {
    time: NormalizedTime<'a>,
    sample_time: NormalizedTime<'a>,
    kind: PropertyKind,
}

impl<'a> From<&'a crate::engine::property::PropertyCaptureRow> for PropertyRow<'a> {
    fn from(row: &'a crate::engine::property::PropertyCaptureRow) -> Self {
        Self {
            time: NormalizedTime::new(row.time.as_str()),
            sample_time: NormalizedTime::new(row.sample_time.as_str()),
            kind: row.kind.into(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ExtractPayloadValue<'a> {
    path: CanonicalPath<'a>,
    value: SampledValue<'a>,
}

impl<'a> From<&'a crate::engine::extract::ExtractPayloadValue> for ExtractPayloadValue<'a> {
    fn from(value: &'a crate::engine::extract::ExtractPayloadValue) -> Self {
        Self {
            path: CanonicalPath::new(value.path.as_str()),
            value: SampledValue::new(value.value.as_str()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ExtractGenericRow<'a> {
    time: NormalizedTime<'a>,
    sample_time: NormalizedTime<'a>,
    source: &'a str,
    payload: Vec<ExtractPayloadValue<'a>>,
}

impl<'a> From<&'a crate::engine::extract::ExtractGenericRow> for ExtractGenericRow<'a> {
    fn from(row: &'a crate::engine::extract::ExtractGenericRow) -> Self {
        Self {
            time: NormalizedTime::new(row.time.as_str()),
            sample_time: NormalizedTime::new(row.sample_time.as_str()),
            source: row.source.as_str(),
            payload: row.payload.iter().map(ExtractPayloadValue::from).collect(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ExtractAhbMapping<'a> {
    path: CanonicalPath<'a>,
}

impl<'a> From<&'a crate::engine::ahb::AhbSignalMapping> for ExtractAhbMapping<'a> {
    fn from(mapping: &'a crate::engine::ahb::AhbSignalMapping) -> Self {
        Self {
            path: CanonicalPath::new(mapping.path.as_str()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ExtractAhbAddressSnapshot<'a> {
    time: NormalizedTime<'a>,
    sample_time: NormalizedTime<'a>,
    transfer: &'a str,
    direction: &'a str,
    payload: BTreeMap<&'a str, SampledValue<'a>>,
}

impl<'a> From<&'a crate::engine::ahb::AhbAddressSnapshot> for ExtractAhbAddressSnapshot<'a> {
    fn from(address: &'a crate::engine::ahb::AhbAddressSnapshot) -> Self {
        Self {
            time: NormalizedTime::new(address.time.as_str()),
            sample_time: NormalizedTime::new(address.sample_time.as_str()),
            transfer: address.transfer.as_str(),
            direction: address.direction.as_str(),
            payload: ahb_payload(&address.payload),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ExtractAhbInitialDataPhase<'a> {
    state: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    address: Option<ExtractAhbAddressSnapshot<'a>>,
}

impl<'a> From<&'a crate::engine::ahb::AhbInitialDataPhase> for ExtractAhbInitialDataPhase<'a> {
    fn from(initial: &'a crate::engine::ahb::AhbInitialDataPhase) -> Self {
        Self {
            state: initial.state.as_str(),
            address: initial
                .address
                .as_ref()
                .map(ExtractAhbAddressSnapshot::from),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ExtractAhbEvent<'a> {
    time: NormalizedTime<'a>,
    sample_time: NormalizedTime<'a>,
    profile: &'a str,
    event: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    transfer: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    direction: Option<&'a str>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    payload: BTreeMap<&'a str, SampledValue<'a>>,
}

impl<'a> From<&'a crate::engine::ahb::AhbEvent> for ExtractAhbEvent<'a> {
    fn from(event: &'a crate::engine::ahb::AhbEvent) -> Self {
        Self {
            time: NormalizedTime::new(event.time.as_str()),
            sample_time: NormalizedTime::new(event.sample_time.as_str()),
            profile: event.profile.as_str(),
            event: event.event.as_str(),
            transfer: event.transfer.as_deref(),
            direction: event.direction.as_deref(),
            payload: ahb_payload(&event.payload),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ExtractAhbData<'a> {
    name: &'a str,
    profile: &'a str,
    issue: &'a str,
    include_stall: bool,
    include_idle: bool,
    include_busy: bool,
    initial_data_phase: ExtractAhbInitialDataPhase<'a>,
    mappings: BTreeMap<&'a str, ExtractAhbMapping<'a>>,
    events: Vec<ExtractAhbEvent<'a>>,
}

impl<'a> From<&'a crate::engine::ahb::AhbData> for ExtractAhbData<'a> {
    fn from(data: &'a crate::engine::ahb::AhbData) -> Self {
        Self {
            name: data.name.as_str(),
            profile: data.profile.as_str(),
            issue: data.issue.as_str(),
            include_stall: data.include_stall,
            include_idle: data.include_idle,
            include_busy: data.include_busy,
            initial_data_phase: ExtractAhbInitialDataPhase::from(&data.initial_data_phase),
            mappings: data
                .mappings
                .iter()
                .map(|mapping| (mapping.standard.as_str(), ExtractAhbMapping::from(mapping)))
                .collect(),
            events: data.events.iter().map(ExtractAhbEvent::from).collect(),
        }
    }
}

fn ahb_payload<'a>(
    payload: &'a [crate::engine::ahb::AhbPayloadValue],
) -> BTreeMap<&'a str, SampledValue<'a>> {
    payload
        .iter()
        .map(|value| {
            (
                value.standard.as_str(),
                SampledValue::new(value.value.as_str()),
            )
        })
        .collect()
}

#[derive(Debug, Serialize)]
pub struct ExtractApbMapping<'a> {
    path: CanonicalPath<'a>,
}

impl<'a> From<&'a crate::engine::apb::ApbSignalMapping> for ExtractApbMapping<'a> {
    fn from(mapping: &'a crate::engine::apb::ApbSignalMapping) -> Self {
        Self {
            path: CanonicalPath::new(mapping.path.as_str()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ExtractApbEvent<'a> {
    time: NormalizedTime<'a>,
    sample_time: NormalizedTime<'a>,
    profile: &'a str,
    event: &'a str,
    direction: &'a str,
    payload: BTreeMap<&'a str, SampledValue<'a>>,
}

impl<'a> From<&'a crate::engine::apb::ApbEvent> for ExtractApbEvent<'a> {
    fn from(event: &'a crate::engine::apb::ApbEvent) -> Self {
        Self {
            time: NormalizedTime::new(event.time.as_str()),
            sample_time: NormalizedTime::new(event.sample_time.as_str()),
            profile: event.profile.as_str(),
            event: event.event.as_str(),
            direction: event.direction.as_str(),
            payload: event
                .payload
                .iter()
                .map(|value| {
                    (
                        value.standard.as_str(),
                        SampledValue::new(value.value.as_str()),
                    )
                })
                .collect(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ExtractApbData<'a> {
    name: &'a str,
    profile: &'a str,
    issue: &'a str,
    pready_mode: &'a str,
    include_wait: bool,
    mappings: BTreeMap<&'a str, ExtractApbMapping<'a>>,
    events: Vec<ExtractApbEvent<'a>>,
}

impl<'a> From<&'a crate::engine::apb::ApbData> for ExtractApbData<'a> {
    fn from(data: &'a crate::engine::apb::ApbData) -> Self {
        Self {
            name: data.name.as_str(),
            profile: data.profile.as_str(),
            issue: data.issue.as_str(),
            pready_mode: data.pready_mode.as_str(),
            include_wait: data.include_wait,
            mappings: data
                .mappings
                .iter()
                .map(|mapping| (mapping.standard.as_str(), ExtractApbMapping::from(mapping)))
                .collect(),
            events: data.events.iter().map(ExtractApbEvent::from).collect(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ExtractAtbMapping<'a> {
    path: CanonicalPath<'a>,
}

impl<'a> From<&'a crate::engine::atb::AtbSignalMapping> for ExtractAtbMapping<'a> {
    fn from(mapping: &'a crate::engine::atb::AtbSignalMapping) -> Self {
        Self {
            path: CanonicalPath::new(mapping.path.as_str()),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExtractAtbEventKind {
    Transfer,
    Flush,
    SyncRequest,
}

impl From<crate::engine::atb::AtbEventKind> for ExtractAtbEventKind {
    fn from(kind: crate::engine::atb::AtbEventKind) -> Self {
        match kind {
            crate::engine::atb::AtbEventKind::Transfer => Self::Transfer,
            crate::engine::atb::AtbEventKind::Flush => Self::Flush,
            crate::engine::atb::AtbEventKind::SyncRequest => Self::SyncRequest,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ExtractAtbEvent<'a> {
    time: NormalizedTime<'a>,
    sample_time: NormalizedTime<'a>,
    profile: &'a str,
    event: ExtractAtbEventKind,
    payload: BTreeMap<&'a str, SampledValue<'a>>,
}

impl<'a> From<&'a crate::engine::atb::AtbEvent> for ExtractAtbEvent<'a> {
    fn from(event: &'a crate::engine::atb::AtbEvent) -> Self {
        Self {
            time: NormalizedTime::new(event.time.as_str()),
            sample_time: NormalizedTime::new(event.sample_time.as_str()),
            profile: event.profile.as_str(),
            event: event.event.into(),
            payload: event
                .payload
                .iter()
                .map(|value| {
                    (
                        value.standard.as_str(),
                        SampledValue::new(value.value.as_str()),
                    )
                })
                .collect(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ExtractAtbData<'a> {
    name: &'a str,
    profile: &'a str,
    issue: &'a str,
    mappings: BTreeMap<&'a str, ExtractAtbMapping<'a>>,
    events: Vec<ExtractAtbEvent<'a>>,
}

impl<'a> From<&'a crate::engine::atb::AtbData> for ExtractAtbData<'a> {
    fn from(data: &'a crate::engine::atb::AtbData) -> Self {
        Self {
            name: data.name.as_str(),
            profile: data.profile.as_str(),
            issue: data.issue.as_str(),
            mappings: data
                .mappings
                .iter()
                .map(|mapping| (mapping.standard.as_str(), ExtractAtbMapping::from(mapping)))
                .collect(),
            events: data.events.iter().map(ExtractAtbEvent::from).collect(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ExtractAxiMapping<'a> {
    path: CanonicalPath<'a>,
}

impl<'a> From<&'a crate::engine::axi::AxiSignalMapping> for ExtractAxiMapping<'a> {
    fn from(mapping: &'a crate::engine::axi::AxiSignalMapping) -> Self {
        Self {
            path: CanonicalPath::new(mapping.path.as_str()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ExtractAxiTransfer<'a> {
    time: NormalizedTime<'a>,
    sample_time: NormalizedTime<'a>,
    profile: &'a str,
    channel: &'a str,
    payload: BTreeMap<&'a str, SampledValue<'a>>,
}

impl<'a> From<&'a crate::engine::axi::AxiTransfer> for ExtractAxiTransfer<'a> {
    fn from(transfer: &'a crate::engine::axi::AxiTransfer) -> Self {
        Self {
            time: NormalizedTime::new(transfer.time.as_str()),
            sample_time: NormalizedTime::new(transfer.sample_time.as_str()),
            profile: transfer.profile.as_str(),
            channel: transfer.channel.as_str(),
            payload: transfer
                .payload
                .iter()
                .map(|value| {
                    (
                        value.standard.as_str(),
                        SampledValue::new(value.value.as_str()),
                    )
                })
                .collect(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ExtractAxiData<'a> {
    name: &'a str,
    profile: &'a str,
    issue: &'a str,
    mappings: BTreeMap<&'a str, ExtractAxiMapping<'a>>,
    transfers: Vec<ExtractAxiTransfer<'a>>,
}

impl<'a> From<&'a crate::engine::axi::AxiData> for ExtractAxiData<'a> {
    fn from(data: &'a crate::engine::axi::AxiData) -> Self {
        Self {
            name: data.name.as_str(),
            profile: data.profile.as_str(),
            issue: data.issue.as_str(),
            mappings: data
                .mappings
                .iter()
                .map(|mapping| (mapping.standard.as_str(), ExtractAxiMapping::from(mapping)))
                .collect(),
            transfers: data
                .transfers
                .iter()
                .map(ExtractAxiTransfer::from)
                .collect(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ExtractAxiStreamMapping<'a> {
    path: CanonicalPath<'a>,
}

impl<'a> From<&'a crate::engine::axistream::AxiStreamSignalMapping>
    for ExtractAxiStreamMapping<'a>
{
    fn from(mapping: &'a crate::engine::axistream::AxiStreamSignalMapping) -> Self {
        Self {
            path: CanonicalPath::new(mapping.path.as_str()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ExtractAxiStreamTransfer<'a> {
    time: NormalizedTime<'a>,
    sample_time: NormalizedTime<'a>,
    profile: &'a str,
    payload: BTreeMap<&'a str, SampledValue<'a>>,
}

impl<'a> From<&'a crate::engine::axistream::AxiStreamTransfer> for ExtractAxiStreamTransfer<'a> {
    fn from(transfer: &'a crate::engine::axistream::AxiStreamTransfer) -> Self {
        Self {
            time: NormalizedTime::new(transfer.time.as_str()),
            sample_time: NormalizedTime::new(transfer.sample_time.as_str()),
            profile: transfer.profile.as_str(),
            payload: transfer
                .payload
                .iter()
                .map(|value| {
                    (
                        value.standard.as_str(),
                        SampledValue::new(value.value.as_str()),
                    )
                })
                .collect(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ExtractAxiStreamData<'a> {
    name: &'a str,
    profile: &'a str,
    issue: &'a str,
    tready_mode: &'a str,
    mappings: BTreeMap<&'a str, ExtractAxiStreamMapping<'a>>,
    transfers: Vec<ExtractAxiStreamTransfer<'a>>,
}

impl<'a> From<&'a crate::engine::axistream::AxiStreamData> for ExtractAxiStreamData<'a> {
    fn from(data: &'a crate::engine::axistream::AxiStreamData) -> Self {
        Self {
            name: data.name.as_str(),
            profile: data.profile.as_str(),
            issue: data.issue.as_str(),
            tready_mode: data.tready_mode.as_str(),
            mappings: data
                .mappings
                .iter()
                .map(|mapping| {
                    (
                        mapping.standard.as_str(),
                        ExtractAxiStreamMapping::from(mapping),
                    )
                })
                .collect(),
            transfers: data
                .transfers
                .iter()
                .map(ExtractAxiStreamTransfer::from)
                .collect(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TopicSummary<'a> {
    id: &'a str,
    title: &'a str,
    description: &'a str,
    section: &'a str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    see_also: Vec<&'a str>,
}

impl<'a> From<&'a crate::docs::TopicSummary> for TopicSummary<'a> {
    fn from(topic: &'a crate::docs::TopicSummary) -> Self {
        Self {
            id: topic.id.as_str(),
            title: topic.title.as_str(),
            description: topic.description.as_str(),
            section: topic.section.as_str(),
            see_also: topic.see_also.iter().map(String::as_str).collect(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct DocsTopicsData<'a> {
    topics: Vec<TopicSummary<'a>>,
}

impl<'a> From<&'a crate::engine::DocsTopicsData> for DocsTopicsData<'a> {
    fn from(data: &'a crate::engine::DocsTopicsData) -> Self {
        Self {
            topics: data.topics.iter().map(TopicSummary::from).collect(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct DocsSearchMatch<'a> {
    topic: TopicSummary<'a>,
    match_kind: DocsMatchKind,
    matched_tokens: usize,
}

impl<'a> From<&'a crate::engine::DocsSearchMatchData> for DocsSearchMatch<'a> {
    fn from(entry: &'a crate::engine::DocsSearchMatchData) -> Self {
        Self {
            topic: TopicSummary::from(&entry.topic),
            match_kind: entry.match_kind.into(),
            matched_tokens: entry.matched_tokens,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum DocsMatchKind {
    IdExact,
    IdPrefix,
    TitleExact,
    TitleOrDescription,
    Heading,
    Body,
}

impl From<crate::docs::MatchKind> for DocsMatchKind {
    fn from(kind: crate::docs::MatchKind) -> Self {
        match kind {
            crate::docs::MatchKind::IdExact => Self::IdExact,
            crate::docs::MatchKind::IdPrefix => Self::IdPrefix,
            crate::docs::MatchKind::TitleExact => Self::TitleExact,
            crate::docs::MatchKind::TitleOrDescription => Self::TitleOrDescription,
            crate::docs::MatchKind::Heading => Self::Heading,
            crate::docs::MatchKind::Body => Self::Body,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct DocsSearchData<'a> {
    query: &'a str,
    matches: Vec<DocsSearchMatch<'a>>,
}

impl<'a> From<&'a crate::engine::DocsSearchData> for DocsSearchData<'a> {
    fn from(data: &'a crate::engine::DocsSearchData) -> Self {
        Self {
            query: data.query.as_str(),
            matches: data.matches.iter().map(DocsSearchMatch::from).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use crate::engine::{CommandData, CommandName, CommandResult, HumanRenderOptions};
    use crate::output_mode::OutputMode;

    use super::OutputEnvelope;

    #[test]
    fn output_envelope_uses_contract_dto_not_engine_display_fields() {
        let result = CommandResult {
            command: CommandName::Value,
            output_mode: OutputMode::Json,
            human_options: HumanRenderOptions::default(),
            data: CommandData::Value(vec![crate::engine::value::ValueSnapshot {
                time: "5ns".to_string(),
                signals: vec![crate::engine::value::ValueSignalValue {
                    display: "sig".to_string(),
                    path: "top.sig".to_string(),
                    value: "1'h1".to_string(),
                }],
            }]),
            diagnostics: Vec::new(),
        };

        let value = serde_json::to_value(
            OutputEnvelope::from_result(&result).expect("result should convert to contract"),
        )
        .expect("contract envelope should serialize");
        assert_eq!(value["data"][0]["signals"][0]["path"], "top.sig");
        assert!(value["data"][0]["signals"][0].get("display").is_none());
    }

    #[test]
    fn docs_topics_omits_empty_see_also() {
        let result = CommandResult {
            command: CommandName::DocsTopics,
            output_mode: OutputMode::Json,
            human_options: HumanRenderOptions::default(),
            data: CommandData::DocsTopics(crate::engine::DocsTopicsData {
                topics: vec![crate::docs::TopicSummary {
                    id: "intro".to_string(),
                    title: "Introduction".to_string(),
                    description: "Start here".to_string(),
                    section: "intro".to_string(),
                    see_also: Vec::new(),
                }],
            }),
            diagnostics: Vec::new(),
        };

        let value: Value = serde_json::to_value(
            OutputEnvelope::from_result(&result).expect("docs topics should convert"),
        )
        .expect("docs topics should serialize");
        assert!(value["data"]["topics"][0].get("see_also").is_none());
    }
}
