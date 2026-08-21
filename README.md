<h1 align="center"><code>wavepeek</code></h1>

<p align="center">
  <img src="docs/wavepeek.svg" alt="wavepeek" width="900">
</p>

<p align="center">
  A CLI for querying RTL waveform dumps.<br>
  Supports signal inspection, clocked Boolean conditions, and generic or AMBA event extraction.
</p>

<p align="center">
  <a href="https://kleverhq.github.io/wavepeek/">Playground</a>
  · <a href="https://kleverhq.github.io/wavepeek/latest/">Documentation</a>
  · <a href="https://kleverhq.github.io/wavepeek/latest/quickstart/">Quickstart</a>
  · <a href="https://github.com/kleverhq/wavepeek/releases">Releases</a>
</p>

<p align="center">
  <a href="https://github.com/kleverhq/wavepeek/actions/workflows/ci.yml"><img src="https://github.com/kleverhq/wavepeek/actions/workflows/ci.yml/badge.svg?branch=main" alt="CI"></a>
  <a href="https://github.com/kleverhq/wavepeek/releases"><img src="https://img.shields.io/github/v/release/kleverhq/wavepeek" alt="GitHub release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/github/license/kleverhq/wavepeek" alt="Apache-2.0 license"></a>
</p>

`wavepeek` runs non-interactive queries over saved waveform files. Its main use cases are LLM-driven debugging and other automated workflows. GUI viewers are better suited to open-ended interactive exploration. `wavepeek` can also be used in scripts that need repeatable queries with limited output.

`wavepeek` is a stateless CLI. Each invocation opens one VCD/FST/FSDB file, runs one query, writes text, JSON, or JSONL, and exits. It starts on demand and does not require a background service. It is not a GUI or TUI waveform viewer. It does not provide real-time waveform streaming, live simulator connections, or waveform comparison.

For example, this command returns the value of `top.data` at `10ns`:

```text
$ wavepeek value --waves dump.vcd --at 10ns --signals top.data
@10ns top.data=8'h0f
```

## Commands

| Task                                                         | Commands                                                                        |
| ------------------------------------------------------------ | ------------------------------------------------------------------------------- |
| Read dump metadata and browse the recorded hierarchy         | `info`, `scope`, `signal`                                                       |
| Read values at a time or on selected events                  | `value`, `change`                                                               |
| Find when a Boolean condition matches, asserts, or deasserts | `property`                                                                      |
| Extract custom handshakes or synchronous events              | `extract generic`                                                               |
| Extract AMBA transfers or phase events                       | `extract axi`, `extract axistream`, `extract ahb`, `extract apb`, `extract atb` |

The protocol extractors support AXI and ACE, AXI-Stream, AHB, APB, and ATB. Their output contains observed channel transfers or protocol phase events with sampled payload and context. They do not check protocol compliance or reconstruct high-level transactions.

## Getting started

Open the [Playground](https://kleverhq.github.io/wavepeek/) to run `wavepeek` in a browser.

To install `wavepeek` on macOS or Linux, run:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://kleverhq.github.io/wavepeek/install.sh | sh
```

The prebuilt binary supports VCD and FST. FSDB support requires a source build on Linux x86_64 and the Synopsys Verdi FSDB Reader SDK.

Alternatively, ask your agent to do the setup:

```text
Install the latest release from https://github.com/kleverhq/wavepeek/releases. Run 'wavepeek skill' to get the skill.
```

Check the version after installation:

```bash
wavepeek --version
```

See the [Quickstart](https://kleverhq.github.io/wavepeek/latest/quickstart/) for Windows, Cargo, source installation, FSDB setup, and further examples.

## Agent skill

Extract the skill package included in the binary:

```bash
wavepeek skill ./wavepeek-skill
```

The package matches the installed `wavepeek` version. Its directory contains `SKILL.md`, offline references, examples, helper scripts, and provenance. Follow your agent harness's instructions to install the directory.

## Documentation

| Topic                        | Link                                                                                                                                                    |
| ---------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Installation and first query | [Quickstart](https://kleverhq.github.io/wavepeek/latest/quickstart/)                                                                                    |
| Command groups               | [Commands](https://kleverhq.github.io/wavepeek/latest/commands/)                                                                                        |
| Hierarchy and signal values  | [Explore dump](https://kleverhq.github.io/wavepeek/latest/explore-dump/) · [Inspect values](https://kleverhq.github.io/wavepeek/latest/inspect-values/) |
| Boolean conditions           | [Evaluate properties](https://kleverhq.github.io/wavepeek/latest/evaluate-properties/)                                                                  |
| Generic and AMBA extraction  | [Extract transfers](https://kleverhq.github.io/wavepeek/latest/extract-transfers/)                                                                      |
| JSON and JSONL output        | [Machine output](https://kleverhq.github.io/wavepeek/latest/machine-output/)                                                                            |
| Syntax and flags             | [CLI reference](https://kleverhq.github.io/wavepeek/latest/cli-reference/)                                                                              |

## Development

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Apache-2.0