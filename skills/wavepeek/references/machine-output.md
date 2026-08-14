# Machine Output Contract

This document is normative for stdout, stderr, JSON-mode behavior, JSONL stream behavior, diagnostics, and exit codes.

## 1. Stdout and stderr

On success, a command writes its main payload to stdout.

- Human-readable mode writes non-fatal diagnostics to stderr as plain text.
- `--json` carries non-fatal diagnostics inside one JSON result.
- `--jsonl` writes one JSON object per stdout line and carries diagnostics as stream records.

If a downstream consumer intentionally closes stdout early, `wavepeek` stops writing and exits successfully. In non-streaming modes, stdout is empty on failure and stderr carries the fatal error. A JSONL failure after `begin` can leave a partial stream without `end`; treat that stream as incomplete.

## 2. JSON envelopes

A successful `--json` command emits `type`, `command`, optional `context`, `data`, and `diagnostics`. `type` is `result`. `data` and `diagnostics` are always arrays. `info` returns one metadata row; an empty query returns `data: []`.

```json
{"type":"result","command":"info","data":[{"time_unit":"1ns","time_start":"0ns","time_end":"10ns"}],"diagnostics":[]}
```

```json
{"type":"result","command":"value","data":[{"time":"5ns","signals":[{"path":"top.clk","value":"1'h1"}]}],"diagnostics":[]}
```

A `signal` row contains the leaf `name`, canonical `path`, path relative to the exact selected scope in `relative_path`, normalized `kind`, and optional `width`. Immediate children use their basename as `relative_path`; descendants retain the child scope path:

```json
{"type":"result","command":"signal","data":[{"name":"valid","path":"top.cpu.valid","relative_path":"cpu.valid","kind":"wire","width":1}],"diagnostics":[]}
```

Protocol extractors put command-wide metadata in `context` and rows directly in `data`:

```json
{"type":"result","command":"extract apb","context":{"name":"apb","profile":"apb4","issue":"E","pready_mode":"mapped","include_wait":false,"mappings":{"pclk":{"path":"top.uart_apb_p_clk_i"},"penable":{"path":"top.uart_apb_penable_o"},"pready":{"path":"top.uart_apb_pready_i"},"psel":{"path":"top.uart_apb_psel_o"},"pwrite":{"path":"top.uart_apb_pwrite_o"}}},"data":[{"time":"5ns","sample_time":"4ns","profile":"apb4","event":"setup","direction":"write","payload":{"pwrite":"1'h1"}}],"diagnostics":[]}
```

Machine-readable `path` fields are canonical; `signal` rows also include scope-relative `relative_path`. Protocol context fields are documented in [Extract command](extract.md); row fields remain command-specific. Waveform commands support JSON envelopes. Unsupported `--json` combinations fail as argument errors.

A diagnostic has `kind`, `message`, and, for warnings and errors, a stable `code`:

```json
{"kind":"warning","code":"WPK-W0002","message":"truncated output to 1 entries (use --max to increase limit)"}
```

`kind` is `info`, `warning`, or `error`. Warning and error codes match `WPK-W####` or `WPK-E####`; information diagnostics omit `code`.

## 3. JSONL streams

Waveform commands support `--jsonl` for incremental consumption. Each stdout line is an independent JSON object:

```jsonl
{"type":"begin","seq":0,"command":"change"}
{"type":"data","seq":1,"data":{"time":"5ns","sample_time":"5ns","signals":[{"path":"top.clk","value":"1'h1"}]}}
{"type":"diagnostic","seq":2,"diagnostic":{"kind":"warning","code":"WPK-W0002","message":"truncated output to 1 entries (use --max to increase limit)"}}
{"type":"end","seq":3,"records":{"data":1,"diagnostics":1}}
```

A successful stream obeys these rules:

- `begin` is first with `seq: 0`; it is the only record containing `command`.
- `seq` increases by one for every record.
- Protocol extractor `begin` records include `context` identical to the JSON result context.
- Each `data.data` is identical to one element of the JSON result `data` array, including the single `info` row.
- `diagnostic.diagnostic` has the same shape as one JSON result diagnostic and may appear between data records.
- `end` is last and its required `records` object counts emitted data and diagnostic records.

`change`, `property`, and extraction rows use `time` for the selected event and `sample_time` for the sampled values. A `change` row's `signals` array follows `--row-values`: full rows contain every requested signal, while delta rows can contain a subset or be empty. The first emitted delta row is always full. Protocol rows repeat `profile`; it must match the begin context. If the process exits non-zero or no final `end` appears, treat the stream as incomplete.

`--json` and `--jsonl` are mutually exclusive. JSONL is available on waveform commands only.

## 4. Diagnostics

Diagnostics do not change the exit code. Common cases are truncation, explicitly disabled limits, and valid queries with no matching rows.

Human-readable diagnostics use:

```text
info: <message>
warning[WPK-W0002]: <message>
error[WPK-E0001]: <message>
```

With `DEBUG=1`, commands may also write JSON debug events to stderr. Debug events are separate from command diagnostics and fatal errors.

## 5. Fatal errors and exit codes

Process-level failures are fail-fast and use:

```text
fatal: <category>: <message>
```

Representative categories include `args`, `file`, `scope`, `signal`, and `expr`. Fatal messages may span multiple stderr lines; missing-signal failures use continuation lines for bounded query-name suggestions or for the explanation that no useful dumped signal exists. JSON and JSONL flags do not wrap these process-level failures, and stdout remains empty when failure occurs before a JSONL `begin` record.

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | User-facing failure such as bad arguments, missing scopes/signals, or invalid expressions |
| `2` | File-level failure such as open, parse, or unsupported-format failures |

Fatal errors are never wrapped in a JSON success envelope.

## 6. Human output flexibility

Human-readable output is intentionally less rigid than machine output. Formatting may improve while preserving the semantic guarantees in [Command model](command-model.md) and command-specific help or tests.
