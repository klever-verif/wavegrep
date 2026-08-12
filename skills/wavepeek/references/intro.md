# Introduction

`wavepeek` is a command-line tool for RTL waveform inspection. It provides deterministic, machine-friendly output and a small set of primitives that compose into repeatable debug recipes.

The product exists to close the waveform access gap for automation. RTL debug usually depends on visually scanning dense temporal data in GUI viewers, but LLM agents, CI jobs, and post-simulation scripts need a textual, deterministic interface instead. `wavepeek` turns waveform dumps into bounded, composable command results that can be reasoned about, piped, and checked automatically.

The primary users are LLM-driven debugging workflows and other automation that need stable output contracts. Humans are still expected to use GUI viewers for open-ended interactive exploration, but `wavepeek` is useful for scripting, repeatable queries, and compact inspections.

## Scope

Default `wavepeek` builds support VCD and FST waveform dumps, hierarchy and signal discovery, explicit-point value sampling, bounded time-range inspection, property checks over event-selected timestamps, extract row generation, and stateless CLI execution with deterministic output. FSDB support is currently Linux x86_64 only and requires installing with the Cargo feature `fsdb` and the Synopsys Verdi FSDB Reader SDK; FSDB-enabled builds support the same waveform command surface for digital bit-vector/integral signals. FSDB real and string value decoding remain unsupported and fail clearly when a command needs those values.

`wavepeek` is not a GUI or TUI waveform viewer. It does not provide real-time waveform streaming, live simulator connections, or waveform diffing and comparison.

## What to expect

`wavepeek` is designed around a few user-visible guarantees:

1. **Machine-friendly output.** Command output, command structure, and error messages should be easy for automation and agents to consume reliably.
2. **Human by default, JSON when requested.** Human-readable output is the default user experience. Stable machine-readable output is opt-in with `--json` where supported.
3. **Composable commands.** Each command does one focused job so scripts and agents can combine commands into repeatable debug recipes.
4. **Deterministic output.** Identical inputs should produce identical observable output.
5. **Stable machine contracts.** JSON and JSONL shapes are documented in [Machine output](reference/machine-output.md) and covered by direct runtime tests, while human-readable output stays intentionally more flexible.
6. **Minimal footprint.** `wavepeek` is stateless, fast to start, and does not require a background service.

## Documentation map

The packaged references are organized by topic type:

- [Command guides](commands/overview.md) help you choose a command family and find exact CLI help.
- [Workflows](workflows/extract-handshake.md) show repeatable task recipes.
- [Troubleshooting](troubleshooting/empty-results.md) explains surprising but valid results and recovery steps.
- [Reference material](reference/command-model.md) defines stable semantics and contracts.

For exact command syntax, defaults, required flags, and examples, use generated help from the installed binary rather than these narrative topics.

## Getting help

Use progressive disclosure when you need help:

- `wavepeek -h` gives compact top-level lookup help.
- `wavepeek --help` gives detailed top-level reference help.
- `wavepeek help <command-path...>` gives detailed help for a top-level or nested command, such as `wavepeek help extract axi`.
- `wavepeek skill <DIRECTORY>` extracts this complete package into a new or empty directory.
