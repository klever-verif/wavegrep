# Machine Output Contract

This document is normative for stdout, stderr, JSON-mode behavior, JSONL stream behavior, diagnostics, and exit codes.

## 1. Stdout and stderr

On success, a command writes its main payload to stdout. Without `--summary`, empty human results use a short command-specific stdout message or retain protocol context with an empty row section.

- Human-readable mode writes non-fatal diagnostics to stderr as plain text.
- `--json` carries non-fatal diagnostics inside one JSON result.
- `--jsonl` writes one JSON object per stdout line and carries diagnostics as stream records.

If a downstream consumer intentionally closes stdout early, `wavepeek` stops writing and exits successfully. Human-mode failures leave stdout empty and write the fatal error to stderr. Machine-mode failures write a fatal object to stdout and do not duplicate it on stderr. With `DEBUG=1`, debug telemetry remains JSON on stderr in every mode.

## 2. JSON envelopes

A successful `--json` command emits `type`, `command`, optional `context`, optional `data`, optional `summary`, and `diagnostics`. `type` is `result`. When present, `data` is an array; `diagnostics` is always an array. `info` returns one metadata row; an empty query returns `data: []`. Commands with numeric or unlimited `--max` always include `summary`.

```json
{"type":"result","command":"info","data":[{"time_unit":"1ns","time_start":"0ns","time_end":"10ns"}],"diagnostics":[]}
```

```json
{"type":"result","command":"value","context":{"scope":"top"},"data":[{"time":"5ns","signals":[{"path":"top.clk","relative_path":"clk","value":"1'h1"}]}],"diagnostics":[]}
```

A `signal` row contains the leaf `name`, canonical `path`, path relative to the exact selected scope in `relative_path`, normalized `kind`, and optional `width`. Immediate children use their basename as `relative_path`; descendants retain the child scope path:

```json
{"type":"result","command":"signal","context":{"scope":"top"},"data":[{"name":"valid","path":"top.cpu.valid","relative_path":"cpu.valid","kind":"wire","width":1}],"summary":{"complete":true,"returned":1,"limit":50,"total":1},"diagnostics":[]}
```

Protocol extractors put command-wide metadata in `context` and rows directly in `data`:

```json
{"type":"result","command":"extract apb","context":{"name":"apb","profile":"apb4","issue":"E","pready_mode":"mapped","include_wait":false,"mappings":{"pclk":{"path":"top.uart_apb_p_clk_i"},"penable":{"path":"top.uart_apb_penable_o"},"pready":{"path":"top.uart_apb_pready_i"},"psel":{"path":"top.uart_apb_psel_o"},"pwrite":{"path":"top.uart_apb_pwrite_o"}}},"data":[{"time":"5ns","sample_time":"4ns","profile":"apb4","event":"setup","direction":"write","payload":{"pwrite":"1'h1"}}],"summary":{"complete":true,"returned":1,"limit":50,"total":1},"diagnostics":[]}
```

Machine-readable `path` fields are canonical. For a flat `[msb:lsb]` projection in `value`, `change`, or `extract generic`, `path` is the canonical source path plus the range and `relative_path` is the scoped relative source path plus the same range; no separate projection field is emitted. Scoped `signal`, `value`, `change`, and `extract generic` results include the exact selected scope in `context.scope`, including when `data` is empty. Their signal or payload rows also include `relative_path`; immediate children use a basename and descendants retain their child scope components. Without `--scope`, `value`, `change`, and `extract generic` omit both `context` and `relative_path` in JSON; their JSONL `begin` records also omit `context`. Protocol context fields are documented in [Extract command](extract.md); row fields remain command-specific. Waveform commands support JSON envelopes. Unsupported `--json` combinations fail as argument errors.

The summary fields describe the selected public result set after filtering:

```json
{"complete":false,"returned":50,"limit":50,"total":null}
```

- `complete` is false only after the command finds another matching public item beyond the numeric limit.
- `returned` is the number of accepted public items, independent of whether rendering suppresses their rows.
- `limit` is the numeric `--max`, or `null` for `--max unlimited`.
- `total` is exact when execution reaches the selected result-set end or the command already knows the full selected count; otherwise it is `null`.

`--max-depth` changes the selected hierarchy set and does not itself make `complete` false. `--max unlimited` scans the selected set to completion, so `complete` is true and `total` equals `returned`.

With `--summary`, JSON keeps `type`, `command`, optional `context`, `summary`, and `diagnostics`, but omits `data`. Selection, filtering, limit enforcement, early stopping, summary values, and diagnostics remain identical to the same invocation without `--summary`.

A diagnostic has `kind`, `message`, and, for warnings and errors, a stable `code`:

```json
{"kind":"warning","code":"WPK-W0002","message":"truncated output to 1 entries (use --max to increase limit)"}
```

`kind` is `info`, `warning`, or `error`. Warning and error codes match `WPK-W####` or `WPK-E####`; information diagnostics omit `code`.

## 3. JSONL streams

Waveform commands support `--jsonl` for incremental consumption. Each stdout line is an independent JSON object:

```jsonl
{"type":"begin","seq":0,"command":"change","context":{"scope":"top"}}
{"type":"data","seq":1,"data":{"time":"5ns","sample_time":"5ns","signals":[{"path":"top.clk","relative_path":"clk","value":"1'h1"}]}}
{"type":"diagnostic","seq":2,"diagnostic":{"kind":"warning","code":"WPK-W0002","message":"truncated output to 1 entries (use --max to increase limit)"}}
{"type":"end","seq":3,"records":{"data":1,"diagnostics":1},"summary":{"complete":false,"returned":1,"limit":1,"total":null}}
```

A successful stream obeys these rules:

- `begin` is first with `seq: 0`; it is the only record containing `command`.
- `seq` increases by one for every record.
- Scoped `signal`, `value`, `change`, and `extract generic` `begin` records include `context.scope` identical to the JSON result context.
- Protocol extractor `begin` records include `context` identical to the JSON result context.
- Each `data.data` is identical to one element of the JSON result `data` array, including the single `info` row.
- `diagnostic.diagnostic` has the same shape as one JSON result diagnostic and may appear between data records.
- `end` is last and its required `records` object counts emitted data and diagnostic records.
- For commands with `--max`, `end.summary` is identical to the JSON result summary.

`change`, `property`, and extraction rows use `time` for the selected event and `sample_time` for the sampled values. A `change` row's `signals` array follows `--row-values`: full rows contain every requested signal, while delta rows can contain a subset or be empty. The first emitted delta row is always full. Protocol rows repeat `profile`; it must match the begin context. If the process exits non-zero or no final `end` appears, treat the stream as incomplete.

With `--summary`, JSONL still emits `begin`, optional context, diagnostics, and `end`, but no `data` records. The terminal `records.data` is therefore zero even when `summary.returned` is nonzero:

```jsonl
{"type":"begin","seq":0,"command":"change"}
{"type":"diagnostic","seq":1,"diagnostic":{"kind":"warning","code":"WPK-W0002","message":"truncated output to 1 entries (use --max to increase limit)"}}
{"type":"end","seq":2,"records":{"data":0,"diagnostics":1},"summary":{"complete":false,"returned":1,"limit":1,"total":null}}
```

`--summary` also suppresses human result rows. Human mode retains command-wide context when present, prints the four summary fields, and writes diagnostics to stderr as usual.

`--json` and `--jsonl` are mutually exclusive selectors and may appear before or after the complete waveform command path. They are recognized only as options before the `--` option terminator, not inside another option's value. When both are present, their argument error uses JSONL. JSONL is available on waveform commands only. Help and version output remain human-readable and ignore either selector.

## 4. Diagnostics

Diagnostics do not change the exit code. Common cases are truncation, unmatched protocol extraction candidates, and ambiguous FSDB signals. Explicitly unlimited limits do not emit diagnostics, and valid queries with no matching rows do not emit an empty-result diagnostic; machine clients distinguish those outcomes through `summary.limit`, available data, and the result counts.

Human-readable diagnostics use:

```text
info: <message>
warning[WPK-W0002]: <message>
error[WPK-E0001]: <message>
```

With `DEBUG=1`, commands may also write JSON debug events to stderr. Debug events are separate from command diagnostics and fatal errors.

## 5. Fatal errors and exit codes

Process-level failures are fail-fast. Human mode writes this form to stderr:

```text
fatal: <category>: <message>
```

`--json` instead writes exactly one flat object to stdout:

```json
{"type":"fatal","code":"WPK-F0002","message":"cannot open 'missing.vcd': No such file or directory"}
```

`--jsonl` writes the same flat fields plus `seq`. A failure before `begin` has `seq: 0`. A failure after records have been written uses the next sequence number, is the last record, and replaces `end`. Fatal records never contain `command`.

```jsonl
{"type":"fatal","seq":0,"code":"WPK-F0001","message":"unrecognized subcommand 'unknown'"}
```

The stable fatal codes are:

| Code | Category |
|------|----------|
| `WPK-F0001` | arguments |
| `WPK-F0002` | file |
| `WPK-F0003` | scope |
| `WPK-F0004` | signal, including a missing signal |
| `WPK-F0005` | expression |
| `WPK-F0006` | internal |
| `WPK-F0007` | unimplemented operation |

The `message` is the error text without the human `fatal: <category>:` prefix. Fatal objects are never wrapped in a JSON success envelope. A successful JSON invocation still emits exactly one result; a successful JSONL stream still ends with exactly one `end`.

Exit codes do not depend on output mode:

| Code | Meaning |
|------|---------|
| `0` | Success, including an intentional downstream broken pipe |
| `1` | User-facing or internal failure such as bad arguments, missing scopes/signals, or invalid expressions |
| `2` | File-level failure such as open, parse, or unsupported-format failures |

If stdout cannot be serialized or written, including a broken pipe, a fatal object is not guaranteed. Ordinary diagnostics retain the forms described above.

## 6. Human output flexibility

Human-readable output is intentionally less rigid than machine output. Formatting may improve while preserving the semantic guarantees in [Command model](command-model.md) and command-specific help or tests.
