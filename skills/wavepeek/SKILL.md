---
name: wavepeek
description: Use the wavepeek CLI to inspect and analyze VCD, FST, or FSDB waveforms, including metadata, hierarchy, values, transitions, properties, and protocol or generic event extraction.
---

# WavePeek

Use `wavepeek` for waveform questions. Treat waveform files as CLI inputs, not as text files to inspect directly.

This extracted package provides version-matched references. Do not infer exact syntax from memory: run `wavepeek --help` or `wavepeek help <command-path...>` before a nontrivial query.

## Start here

1. Run `wavepeek info --waves <FILE> --json` to confirm format, bounds, and time unit.
2. Discover unknown hierarchy with `scope`, then signals within a selected scope with `signal`.
3. Choose the command matching the question:
   - `value` for state at explicit timestamps;
   - `change` for displayed value transitions;
   - `property` for timestamps where a Boolean condition matches or changes state;
   - `extract` for every matching event or transfer with payload values.
4. Use `--json` for scripts and agent-side processing, and inspect diagnostics before trusting results.

Read [Command overview](references/commands/overview.md) for command selection and [Help command](references/commands/help.md) for exact help discovery.

## Safety and query discipline

- Do not read `.fst` or `.fsdb` with generic text or binary tools. Avoid raw `.vcd` reads too; dumps can be large and timing-sensitive.
- Keep output bounded with filters, focused signal lists, explicit time windows, and row limits.
- Use canonical full signal paths without `--scope`; with `--scope`, use names relative to that scope throughout the query. Do not mix naming modes.
- Time tokens require explicit units. Use `info` to learn dump bounds and precision.
- For synchronous RTL, separate the sampling event from the tested condition: use `--on 'posedge <clock>'` and put the condition in `--eval` or `--when`.
- Edge-triggered queries commonly evaluate pre-edge values. Use each row's `sample_time` for follow-up sampling unless same-edge dump state is intentional.
- Derive event and transaction counts from `extract` or `property --capture match`, not from `change`; repeated events with unchanged displayed values are intentionally collapsed by `change`.
- For every JSON result, inspect the top-level `diagnostics` before using counts or conclusions.

## References

Read only the references needed for the current task:

- Commands: [info](references/commands/info.md), [scope](references/commands/scope.md), [signal](references/commands/signal.md), [value](references/commands/value.md), [change](references/commands/change.md), [property](references/commands/property.md), and [extract](references/commands/extract.md).
- Shared semantics: [command model](references/reference/command-model.md), [expression language](references/reference/expression-language.md), and [machine output](references/reference/machine-output.md).
- Workflows: [extract a clocked handshake](references/workflows/extract-handshake.md) and [find the first change](references/workflows/find-first-change.md).
- Troubleshooting: [empty results](references/troubleshooting/empty-results.md), [clock-edge sampling](references/troubleshooting/clock-edge-sampling.md), [scoped versus canonical names](references/troubleshooting/scoped-vs-canonical-names.md), [time tokens and alignment](references/troubleshooting/time-tokens-and-alignment.md), and [unsupported signal encodings](references/troubleshooting/unsupported-signal-encodings.md).
- Large dumps: [waveform performance](references/reference/waveform-performance.md).

## Final checks

Before reporting a result:

- confirm the queried interval covers the requested range;
- confirm no truncation or other diagnostic was ignored;
- use `sample_time` consistently when correlating rows;
- state the scope and clock when they affect the conclusion;
- for protocol completion claims, identify the channel or event that proves completion.
