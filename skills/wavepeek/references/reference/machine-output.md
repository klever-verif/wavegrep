# Machine Output Contract

This document is normative for stdout, stderr, JSON-mode behavior, JSONL stream behavior, diagnostics, and exit codes.

## 1. Stdout and stderr

On success, a command writes its main payload to stdout.

- Human-readable mode writes non-fatal diagnostics to stderr as plain text.
- `--json` carries non-fatal diagnostics inside one JSON result.
- `--jsonl` writes one JSON object per stdout line and carries diagnostics as stream records.

If a downstream consumer intentionally closes stdout early, `wavepeek` stops writing and exits successfully. In non-streaming modes, stdout is empty on failure and stderr carries the fatal error. A JSONL failure after `begin` can leave a partial stream without `end`; treat that stream as incomplete.

## 2. JSON envelopes

A successful `--json` command emits `command`, `data`, and `diagnostics`. `data` may be an object or a list.

Object payload (`info`):

```json
{"command":"info","data":{"time_unit":"1ns","time_start":"0ns","time_end":"10ns"},"diagnostics":[]}
```

List payload (`value`):

```json
{"command":"value","data":[{"time":"5ns","signals":[{"path":"top.clk","value":"1'h1"}]}],"diagnostics":[]}
```

Event payload (`extract apb`):

```json
{"command":"extract apb","data":{"name":"apb","profile":"apb4","issue":"E","pready_mode":"mapped","include_wait":false,"mappings":{"pclk":{"path":"top.uart_apb_p_clk_i"},"penable":{"path":"top.uart_apb_penable_o"},"pready":{"path":"top.uart_apb_pready_i"},"psel":{"path":"top.uart_apb_psel_o"},"pwrite":{"path":"top.uart_apb_pwrite_o"}},"events":[{"time":"5ns","sample_time":"4ns","profile":"apb4","event":"setup","direction":"write","payload":{"pwrite":"1'h1"}}]},"diagnostics":[]}
```

Transfer payload (`extract axi`):

```json
{"command":"extract axi","data":{"name":"axi","profile":"axi4-lite","issue":"H.c","mappings":{"aclk":{"path":"top.clk"},"awready":{"path":"top.axi_aw_ready_i"},"awvalid":{"path":"top.axi_aw_valid_o"}},"transfers":[{"time":"5ns","sample_time":"4ns","profile":"axi4-lite","channel":"aw","payload":{}}]},"diagnostics":[]}
```

Machine-readable paths are canonical. Protocol extraction payloads retain the context fields documented in [Extract command](../commands/extract.md); rows contain only mapped observations. Waveform commands support JSON envelopes. Unsupported `--json` combinations fail as argument errors.

A diagnostic has `kind`, `message`, and, for warnings and errors, a stable `code`:

```json
{"kind":"warning","code":"WPK-W0002","message":"truncated output to 1 entries (use --max to increase limit)"}
```

`kind` is `info`, `warning`, or `error`. Warning and error codes match `WPK-W####` or `WPK-E####`; information diagnostics omit `code`.

## 3. JSONL streams

Waveform commands support `--jsonl` for incremental consumption. Each stdout line is an independent JSON object:

```jsonl
{"type":"begin","seq":0,"command":"change"}
{"type":"item","seq":1,"command":"change","item":{"time":"5ns","sample_time":"5ns","signals":[{"path":"top.clk","value":"1'h1"}]}}
{"type":"diagnostic","seq":2,"command":"change","diagnostic":{"kind":"warning","code":"WPK-W0002","message":"truncated output to 1 entries (use --max to increase limit)"}}
{"type":"end","seq":3,"command":"change","summary":{"status":"ok","items":1,"diagnostics":1,"truncated":true}}
```

A successful stream obeys these rules:

- `begin` is first with `seq: 0`.
- `seq` increases by one for every record and `command` stays constant.
- Protocol extractor `begin` records include the matching protocol context.
- `item` carries the corresponding JSON row, event, transfer, or `info` object.
- `diagnostic` carries the same diagnostic shape used by `--json`.
- `end` is last and reports status, item count, diagnostic count, and truncation.

`change`, `property`, and extraction rows use `time` for the selected event and `sample_time` for the sampled values. Protocol rows repeat `profile`; it must match the begin context. If the process exits non-zero or no final `end` appears, treat the stream as incomplete.

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

Representative categories include `args`, `file`, `scope`, `signal`, and `expr`.

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | User-facing failure such as bad arguments, missing scopes/signals, or invalid expressions |
| `2` | File-level failure such as open, parse, or unsupported-format failures |

Fatal errors are never wrapped in a JSON success envelope.

## 6. Human output flexibility

Human-readable output is intentionally less rigid than machine output. Formatting may improve while preserving the semantic guarantees in [Command model](command-model.md) and command-specific help or tests.
