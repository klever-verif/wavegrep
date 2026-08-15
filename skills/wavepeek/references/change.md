# Change command

Use `change` when explicit point snapshots are not enough and you need an event-aligned table or a compact transition log.

`change` scans an inclusive time window and samples `--signals` at timestamps selected by the required `--on` event expression. By default, every selected event that can be sampled produces a row containing every requested signal.

Two independent options control the table:

| Options | Rows | Values in each row |
|---|---|---|
| `--row-mode dense --row-values full` | Every selected event that can be sampled | All requested signals |
| `--row-mode dense --row-values delta` | Every selected event that can be sampled | First row is full; later rows contain changed signals only |
| `--row-mode sparse --row-values full` | Selected samples that changed | All requested signals |
| `--row-mode sparse --row-values delta` | Selected samples that changed | First emitted row is full; later rows contain changed signals only |

`dense` and `full` are the defaults. A later dense/delta row can have no signal values when the selected sample matches the previous selected sample. In human output that row is only `@<time>` plus `sample@<time>` when pre-edge sampling uses a different timestamp.

Sparse comparisons use the previous selected sample, not the preceding dump timestamp. State advances on every selected sample even when sparse mode suppresses its row. `--max` counts rows after this filtering.

`--on` is intentionally a SystemVerilog-style event-expression surface. Treat it as a practical CLI spelling of the same concepts you would use in `@(...)`: named events, `posedge`/`negedge`/`edge`, `*` for any tracked change, unions with `or` or `,`, and `iff` for gating. For the full shipped syntax and semantics, see [Expression language](expression-language.md).

Use `--on '*' --sample-mode native` when you want any change in the tracked signal set to select a sample. Use an edge such as `--on 'posedge clk'` for an event-aligned cycle table.

For exact syntax and flags, run `wavepeek help change`.

## Project flat vector ranges

A `--signals` entry may end in one static decimal `[msb:lsb]`. The projection indexes the normalized sampled value, with bit zero at the right, and `[n:n]` selects one bit. Exact waveform paths are resolved first; `[n]` remains ordinary waveform path syntax. Dynamic, reversed, out-of-range, chained, and multidimensional selections are rejected.

Projected entries are independent request positions. Overlapping ranges, duplicate ranges, and a range beside its complete source are preserved in order. Sparse and delta comparisons use each projected value rather than the complete source. For `--on '*'`, a source change outside every requested projection does not select a row; adding the complete source makes every source change relevant again. Explicit named and edge terms in `--on` continue to evaluate their complete expression signal.

## Start with a short window and a focused signal list

This is the fastest way to answer "what changed here?":

```text
$ wavepeek change --waves /opt/rtl-artifacts/picorv32_test_ez_vcd.fst \
    --scope testbench.uut \
    --signals cpu_state,mem_valid,mem_ready,trap \
    --from 1010000ps --to 1040000ps \
    --on '*' --sample-mode native --max 10
@1020000ps cpu_state=8'h40 mem_valid=1'h1 mem_ready=1'h0 trap=1'h0
@1030000ps cpu_state=8'h40 mem_valid=1'h1 mem_ready=1'h1 trap=1'h0
@1040000ps cpu_state=8'h40 mem_valid=1'h0 mem_ready=1'h0 trap=1'h0
```

Use this as a basic pattern when you already know the scope and just need raw transition points.

## Trigger on one signal, but print several

`--on` decides when to sample. `--signals` decides what to print.

If you already know SystemVerilog event controls, read `--on` the same way: `mem_valid` means any change, `posedge mem_valid` means rising edges only, `*` means any change in the tracked set, and `iff` gates an event term without changing what gets printed.

If you care about every change of `mem_valid`, use the signal itself as the event:

```text
$ wavepeek change --waves /opt/rtl-artifacts/picorv32_test_ez_vcd.fst \
    --scope testbench.uut \
    --signals cpu_state,mem_valid,mem_ready,trap \
    --from 1000000ps --to 1040000ps \
    --on mem_valid --sample-mode native --max 10
@1020000ps cpu_state=8'h40 mem_valid=1'h1 mem_ready=1'h0 trap=1'h0
@1040000ps cpu_state=8'h40 mem_valid=1'h0 mem_ready=1'h0 trap=1'h0
```

Named event `mem_valid` means any change of that signal, not only the rising edge. Plain signal and wildcard triggers use dump-native sampling, so pass `--sample-mode native` with them.

## Keep only the edge you care about

If the deassert edge is noise, switch to an edge trigger:

```text
$ wavepeek change --waves /opt/rtl-artifacts/picorv32_test_ez_vcd.fst \
    --scope testbench.uut \
    --signals cpu_state,mem_valid,mem_ready,trap \
    --from 1000000ps --to 1040000ps \
    --on "posedge mem_valid" --sample-mode native --max 10
@1020000ps cpu_state=8'h40 mem_valid=1'h1 mem_ready=1'h0 trap=1'h0
```

This is usually the cleanest way to inspect request starts, handshake assertions, enables, and state-entry pulses when you want the dump value at the trigger timestamp. For clocked RTL/SVA-style inspection, use the owning clock edge and the default pre-edge sampling instead.

## Sample on clock edges only while a condition is true

When combinational chatter is irrelevant, gate sampling with `iff`:

```text
$ wavepeek change --waves /opt/rtl-artifacts/picorv32_test_ez_vcd.fst \
    --scope testbench.uut \
    --signals cpu_state,mem_valid,mem_ready,trap \
    --from 1010000ps --to 1040000ps \
    --on "posedge clk iff mem_valid" --max 10
@1030000ps sample@1029999ps cpu_state=8'h40 mem_valid=1'h1 mem_ready=1'h0 trap=1'h0
```

This means: sample on `posedge clk`, but only on cycles where `mem_valid` is true. Dense mode keeps every gated clock event. Add `--row-mode sparse` when unchanged sampled values should suppress a row.

## Choose native or pre-edge sampling on clock edges

By default, `change` uses pre-edge value sampling. A row selected by an edge-only trigger such as `--on 'posedge clk'` keeps the row timestamp at the edge, but prints the selected `--signals` values from immediately before that edge. Human output shows `sample@<time>` when the sampled-value timestamp differs from the trigger timestamp:

```text
$ wavepeek change --waves path/to/dump.vcd --scope top \
    --signals state,valid \
    --on 'posedge clk'
@25ns sample@24999ps state=3'h2 valid=1'h1
```

Pre-edge sampling is accepted only with an explicit edge-only `--on`: `posedge`, `negedge`, or `edge`, optionally with `iff`. The trigger edge detection and any `iff` guard still use dump-native values at the edge timestamp; only the displayed signal values move to the pre-edge sample point. JSON and JSONL rows always include both `time` and `sample_time`; use `sample_time` for follow-up `value --at` checks.

Use `--sample-mode native` for wildcard, plain-signal, or mixed triggers, or when you intentionally want values from the same dump timestamp as the selected event. Use the default pre-edge mode when a value updated by nonblocking assignment at a clock edge appears one clock early compared with an RTL assertion or simulator log. See [Clock-edge sampling](clock-edge-sampling.md) for diagrams and trade-offs.

## Use scope-relative names or full canonical paths

With `--scope`, use short relative names or canonical paths inside the scope. Without it, pass canonical paths directly:

```text
$ wavepeek change --waves /opt/rtl-artifacts/picorv32_test_ez_vcd.fst \
    --signals testbench.uut.cpu_state,testbench.uut.mem_valid,testbench.uut.mem_ready,testbench.uut.trap \
    --from 0ps --to 20000ps \
    --on '*' --sample-mode native --max 20
@10000ps testbench.uut.cpu_state=8'h40 testbench.uut.mem_valid=1'h0 testbench.uut.mem_ready=1'h0 testbench.uut.trap=1'h0
```

If you like scoped input but still want canonical names in human output, add `--abs`:

```text
$ wavepeek change --waves /opt/rtl-artifacts/picorv32_test_ez_vcd.fst \
    --scope testbench.uut \
    --signals cpu_state,mem_valid,mem_ready,trap \
    --from 0ps --to 20000ps \
    --on '*' --sample-mode native --abs
@10000ps testbench.uut.cpu_state=8'h40 testbench.uut.mem_valid=1'h0 testbench.uut.mem_ready=1'h0 testbench.uut.trap=1'h0
```

## Use JSON for scripts and agents

`--json` keeps canonical paths and includes diagnostics in the payload:

```text
$ wavepeek change --waves /opt/rtl-artifacts/picorv32_test_ez_vcd.fst \
    --scope testbench.uut \
    --signals cpu_state,mem_valid,mem_ready,trap \
    --from 1010000ps --to 1040000ps \
    --on '*' --sample-mode native --json
{"type":"result","command":"change","context":{"scope":"testbench.uut"},"data":[{"time":"1020000ps","sample_time":"1020000ps","signals":[{"path":"testbench.uut.cpu_state","relative_path":"cpu_state","value":"8'h40"},{"path":"testbench.uut.mem_valid","relative_path":"mem_valid","value":"1'h1"},{"path":"testbench.uut.mem_ready","relative_path":"mem_ready","value":"1'h0"},{"path":"testbench.uut.trap","relative_path":"trap","value":"1'h0"}]},{"time":"1030000ps","sample_time":"1030000ps","signals":[{"path":"testbench.uut.cpu_state","relative_path":"cpu_state","value":"8'h40"},{"path":"testbench.uut.mem_valid","relative_path":"mem_valid","value":"1'h1"},{"path":"testbench.uut.mem_ready","relative_path":"mem_ready","value":"1'h1"},{"path":"testbench.uut.trap","relative_path":"trap","value":"1'h0"}]},{"time":"1040000ps","sample_time":"1040000ps","signals":[{"path":"testbench.uut.cpu_state","relative_path":"cpu_state","value":"8'h40"},{"path":"testbench.uut.mem_valid","relative_path":"mem_valid","value":"1'h0"},{"path":"testbench.uut.mem_ready","relative_path":"mem_ready","value":"1'h0"},{"path":"testbench.uut.trap","relative_path":"trap","value":"1'h0"}]}],"summary":{"complete":true,"returned":3,"limit":50,"total":3},"diagnostics":[]}
```

## Use JSONL for large ranges and incremental consumers

`--jsonl` streams one JSON object per line. The first line is `begin`, each change snapshot is a `data` record, diagnostics are `diagnostic` records, and a successful stream ends with `end`:

```text
$ wavepeek change --waves /opt/rtl-artifacts/picorv32_test_ez_vcd.fst \
    --scope testbench.uut \
    --signals cpu_state,mem_valid \
    --from 1010000ps --to 1040000ps \
    --on '*' --sample-mode native --jsonl
{"type":"begin","seq":0,"command":"change","context":{"scope":"testbench.uut"}}
{"type":"data","seq":1,"data":{"time":"1020000ps","sample_time":"1020000ps","signals":[{"path":"testbench.uut.cpu_state","relative_path":"cpu_state","value":"8'h40"},{"path":"testbench.uut.mem_valid","relative_path":"mem_valid","value":"1'h1"}]}}
{"type":"data","seq":2,"data":{"time":"1040000ps","sample_time":"1040000ps","signals":[{"path":"testbench.uut.cpu_state","relative_path":"cpu_state","value":"8'h40"},{"path":"testbench.uut.mem_valid","relative_path":"mem_valid","value":"1'h0"}]}}
{"type":"end","seq":3,"records":{"data":2,"diagnostics":0},"summary":{"complete":true,"returned":2,"limit":50,"total":2}}
```

Use this mode for automation that wants to consume rows while the scan is still running. Require a final `end` record and inspect `end.summary.complete` before treating the selected result set as complete. Add `--summary` to suppress data rows while retaining diagnostics and the terminal summary.

## Watch for bounded-output diagnostics

If `--max` truncates the result, the command still succeeds and emits a diagnostic:

```text
$ wavepeek change --waves /opt/rtl-artifacts/picorv32_test_ez_vcd.fst \
    --scope testbench.uut \
    --signals cpu_state,mem_valid,mem_ready,trap \
    --from 1000000ps --to 11000000ps \
    --on "posedge clk" --sample-mode native --max 3
@1020000ps cpu_state=8'h40 mem_valid=1'h1 mem_ready=1'h0 trap=1'h0
@1030000ps cpu_state=8'h40 mem_valid=1'h1 mem_ready=1'h1 trap=1'h0
@1040000ps cpu_state=8'h40 mem_valid=1'h0 mem_ready=1'h0 trap=1'h0
warning[WPK-W0002]: truncated output to 3 entries (use --max to increase limit)
```

`--max unlimited` disables truncation without emitting a diagnostic. In machine output, `summary.limit` is `null`.

## Non-obvious behavior

- VCD and FST work in default builds. FSDB works only in binaries built with the `fsdb` Cargo feature and a local Verdi FSDB Reader SDK. FSDB `change` supports digital bit-vector/integral signals, including raw event triggers when the FSDB contains event occurrences; unsupported real or string values fail with a `signal` error.
- `--from` and `--to` are inclusive. Dense mode emits a matching event at `--from`; sparse mode uses the `--from` sample only as its comparison baseline.
- In pre-edge mode, an event is skipped when no representable point exists before it. This includes a range-start event when the dump has no earlier sample point.
- `--on` guarantees a row only in dense mode. Sparse mode suppresses a selected sample if none of the requested `--signals` changed from the previous selected sample.
- `--sample-mode pre-edge` is the default and requires an explicit edge-only trigger. Use `--sample-mode native` for wildcard, plain-signal, or mixed triggers and for same-timestamp dump sampling.
- JSON and JSONL rows always include `sample_time`. In native mode it equals `time`; in pre-edge mode it is the timestamp whose values were printed.
- In scoped mode, `--signals` and `--on` accept relative names and canonical paths inside the scope, including both forms in one request. `--signals` also accepts one trailing static `[msb:lsb]` projection. Without `--scope`, use canonical full paths.
- Empty output is valid. If the query is well-formed but nothing matched, human output prints `no change rows found in selected time range`. JSON returns `data: []` and an empty `diagnostics` array; JSONL emits no data or diagnostic records. The result summary reports zero returned rows.

When a query keeps coming back empty, widen one dimension at a time: start with the time window, then the trigger, then the signal list.