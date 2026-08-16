# Quickstart

This page takes you from installation to your first `wavepeek` query.

## Install

Install a prebuilt binary. Prebuilt binaries support only VCD/FST.

To install a specific version, see the instructions for the required release on [Releases](https://github.com/kleverhq/wavepeek/releases).

Otherwise, follow the instructions below to install the latest version.

macOS and Linux:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://kleverhq.github.io/wavepeek/install.sh | sh
```

Windows PowerShell:

```bash
powershell -ExecutionPolicy Bypass -c "irm https://kleverhq.github.io/wavepeek/install.ps1 | iex"
```

Cargo is also available as a fallback:

```bash
cargo install wavepeek
```

Or install from source (clone the repository first):

```bash
cargo install --path .
```

## Install with FSDB

FSDB support is source-only and requires Linux x86_64 and the Synopsys Verdi FSDB Reader SDK. Before running the command below, set `VERDI_HOME` to a valid Verdi installation.

```bash
cargo install wavepeek --features fsdb
```

To check whether FSDB is enabled in the current binary, use `--help`:

```text
$ wavepeek --help | grep FSDB
- FSDB - enabled
```

## Setup skill

The `wavepeek` skill is bundled into the binary. Besides `SKILL.md`, it includes reference documentation, examples and helper scripts.
To extract the skill bundle, use:

```bash
wavepeek skill <OUTPUT-DIRECTORY>
```

Alternatively, copy and paste this into your agent:

```text
Check whether `wavepeek` is installed:

wavepeek --version

If that succeeds, run:

wavepeek skill ./wavepeek-skill

`wavepeek skill <DIRECTORY>` extracts the complete, version-matched skill package into a new or empty directory. Install the entire extracted package according to your agent harness's skill rules. Replace older version of the skill entirely if current version is newer.
```

## First run

Start with a waveform dump. You can download example `.fst` dumps from [`rtl-artifacts` releases](https://github.com/kleverhq/rtl-artifacts/releases). Pick any you like.

Check the dump bounds and time unit:

```bash
wavepeek info --waves your_waves.fst
```

Discover the hierarchy:

```bash
wavepeek scope --waves your_waves.fst --tree
```

## Next steps

- [Commands](commands.md) - command model and common conventions.
- [Waveforms](waveforms.md) - formats, conventions, performance, and FSDB support.
- [Output](output.md) - machine output details, JSON envelopes, and JSONL objects.
- [Basic usage](explore-dump.md) - main usage scenarios and short practical examples.
