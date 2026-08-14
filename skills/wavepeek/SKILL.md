---
name: wavepeek
description: Use the wavepeek CLI to inspect and analyze VCD, FST, or FSDB waveforms, including metadata, hierarchy, values, transitions, properties, and protocol or generic event extraction.
---

# WavePeek

Use `wavepeek` for waveform questions. Treat waveform files as CLI inputs, not as text files to inspect directly.

This extracted package provides version-matched references. Do not infer exact syntax from memory: run `wavepeek --help` or `wavepeek help <command-path...>` before a nontrivial query.

## Start here

1. Run `wavepeek info --waves <FILE> --json` to confirm bounds and time unit.
2. Discover unknown hierarchy with `scope`, then signals within a selected scope with `signal`.
3. Choose the command matching the question:
   - `value` for state at explicit timestamps;
   - `change` for event-aligned signal tables or sparse value transitions;
   - `property` for timestamps where a Boolean condition matches or changes state;
   - `extract` for every matching event or transfer with payload values.
4. Use `--json` for scripts and agent-side processing, and inspect diagnostics before trusting results.

Read [Command overview](references/overview.md) for command selection and [Help command](references/help.md) for exact help discovery.

## Safety and query discipline

- Do not read `.fst` or `.fsdb` with generic text or binary tools. Avoid raw `.vcd` reads too; dumps can be large and timing-sensitive.
- Keep output bounded with filters, focused signal lists, explicit time windows, and row limits. Use `--summary` when you need completeness metadata without result rows.
- For `value`, `change`, `property`, and `extract generic`, use canonical paths without `--scope`; with `--scope`, relative and in-scope canonical names may be mixed. Protocol extract mappings remain scope-relative.
- Time tokens require explicit units. Use `info` to learn dump bounds and precision.
- Write hexadecimal expression literals in SystemVerilog form, such as `64'h10` or `128'h0011...`; `0x...` and `64h10` are invalid.
- For synchronous RTL, separate the sampling event from the tested condition: use `--on 'posedge <clock>'` and put the condition in `--eval` or `--when`.
- Edge-triggered queries commonly evaluate pre-edge values. Use each row's `sample_time` for follow-up sampling unless same-edge dump state is intentional.
- For repeated event counts with raw signal payloads, use dense `change`; for derived conditions use `property --capture match`, and for protocol semantics use `extract`.
- For JSON results from commands with `--max`, inspect both top-level `summary` and `diagnostics` before using counts or conclusions. `complete: false` means the returned rows do not cover the selected result set.

## References

Read only the references needed for the current task:

- Commands: [info](references/info.md), [scope](references/scope.md), [signal](references/signal.md), [value](references/value.md), [change](references/change.md), [property](references/property.md), and [extract](references/extract.md).
- Shared semantics: [command model](references/command-model.md), [expression language](references/expression-language.md), and [machine output](references/machine-output.md).
- Workflows: [extract a clocked handshake](references/extract-handshake.md) and [find the first change](references/find-first-change.md).
- Troubleshooting: [empty results](references/empty-results.md), [clock-edge sampling](references/clock-edge-sampling.md), [scoped versus canonical names](references/scoped-vs-canonical-names.md), [time tokens and alignment](references/time-tokens-and-alignment.md), and [unsupported signal encodings](references/unsupported-signal-encodings.md).
- Large dumps: [waveform performance](references/waveform-performance.md).

## Final checks

Before reporting a result:

- confirm the queried interval covers the requested range;
- when `summary` is present, confirm `summary.complete` is true before claiming exhaustive coverage, and do not ignore truncation or other diagnostics;
- use `sample_time` consistently when correlating rows;
- state the scope and clock when they affect the conclusion;
- for protocol completion claims, identify the channel or event that proves completion.
