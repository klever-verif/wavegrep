# Product Requirements Document: wavepeek

## 1. Overview

### 1.1 Product Vision
wavepeek is a command-line tool for RTL waveform inspection. It provides deterministic, machine-friendly output and a minimal set of primitives that compose into repeatable debug recipes.

### 1.2 Problem Statement

**The waveform access gap for LLM agents**

In RTL design and verification, waveforms are the primary debugging artifact. Unlike software where logs, stack traces, and debugger output are mostly textual and sequential, RTL debugging requires analyzing dense temporal data across hierarchical module structures. Waveforms are a high-bandwidth information channel — humans process them visually, leveraging the eye's capacity for pattern recognition.

**Why waveforms matter:**
- Bug reports come with attached dumps (reproducing from scratch can take days)
- Engineers ask questions about waveforms and want to delegate analysis
- Waveforms are the de-facto artifact for RTL debugging — without them, little can be done

**The gap:** There is no tool to "grep" waveforms — to extract signal information, values, and temporal relationships via short, composable commands. GUI viewers (GTKWave) work for humans but cannot be automated. This leaves LLM agents blind to simulation results.

### 1.3 Target Users

**Primary: LLM agents**
- Agentic systems for RTL debugging and verification
- Need structured, deterministic output for reasoning
- Require composable primitives for tool chaining

**Secondary: CI/CD and automation**
- Automated regression pipelines
- Post-simulation analysis scripts
- Waveform-based assertions in CI

**Non-target: Interactive human debugging**
- Humans are not the primary audience
- The tool can be used for scripting and quick queries
- Interactive waveform exploration will still be done in GUI viewers

### 1.4 Design Principles

1. **LLM-first** — Output formats, command structure, and error messages are designed for machine consumption
2. **Self-documenting I/O** — Commands read as unambiguous descriptions of what they do. Output can be interpreted without external context (name=value pairs, no positional-only columns, no short aliases)
3. **Composable commands** — Unix philosophy: do one thing well, combine via pipes. Command names are unambiguous first, short second
4. **Deterministic output** — Same input always produces same output (no timestamps, random IDs, etc.)
5. **Stable formats** — JSON as primary machine-readable format with versioned schema
6. **Minimal footprint** — Fast startup, low memory, no background processes

---

## 2. Scope

### 2.1 In Scope
- VCD/FST dump file support
- Signal discovery: list, search, hierarchy navigation
- Value extraction over time ranges
- Event search (find time when condition holds)
- Stateless CLI (no sessions, no caching, no background processes)

### 2.2 Out of Scope
- GUI/TUI waveform viewer
- Real-time waveform streaming
- Live simulator connections
- Waveform diffing/comparison

### 2.3 Future Considerations
- MCP server for LLM agent integration
- Other formats (FSDB, VPD, WLF, etc.)

---

## 3. Functional Requirements

### 3.1 Supported File Formats

- VCD (Value Change Dump)
- FST (Fast Signal Trace)

### 3.2 CLI Commands

#### General conventions

- **No positional arguments.** All arguments are named flags for self-documenting commands.
  The dump file is always passed via `--waves`.
- **Bounded output.** Commands that return lists always have `--max`
  with a reasonable default (typically 50) to avoid flooding LLM context.
- **Bounded recursion.** Recursive commands have `--max-depth` with a default of 5.
- **Output format.** Default output is TSV without header (self-documenting fields).
  All commands support `--json` flag for JSON array output.
- **Time format.** All time values require explicit units: `fs`, `ps`, `ns`, `us`, `ms`, `s`.
  Bare numbers without units are rejected.
- **Time ranges.** Commands that operate on time ranges use `--from` and `--to`.
  Both are optional: `--from` + `--to` defines a window, only `--from` means
  from that point to end of dump, only `--to` means from start to that point,
  neither means the entire dump.

#### 3.2.1 `info` — Dump metadata

Outputs basic metadata about the waveform dump.

```
wavepeek info --waves <file> [--json]
```

**Parameters:**
| Parameter | Default | Description |
|-----------|---------|-------------|
| `--waves <file>` | required | Path to VCD/FST file |
| `--json` | off | Output as JSON object instead of TSV |

**Behavior:**
- Outputs dump metadata as key-value pairs (one per line)
- JSON mode outputs a single object (not array)

**Output fields:**
| Key | Description |
|-----|-------------|
| `time_unit` | Time unit of the dump (e.g., `1ns`) |
| `time_precision` | Time precision (e.g., `1ps`) |
| `time_start` | Start time of the dump (in `time_unit`) |
| `time_end` | End time of the dump (in `time_unit`) |

**Examples:**
```bash
# Get dump metadata
wavepeek info --waves dump.vcd

# Output:
# time_unit       1ns
# time_precision  1ps
# time_start      0ns
# time_end        10us

# As JSON
wavepeek info --waves dump.vcd --json
```

#### 3.2.2 `tree` — Hierarchy exploration

Outputs a flat list of module instances, recursively traversing the hierarchy.

```
wavepeek tree --waves <file> [--max <n>] [--max-depth <n>] [--filter <regex>] [--json]
```

**Parameters:**
| Parameter | Default | Description |
|-----------|---------|-------------|
| `--waves <file>` | required | Path to VCD/FST file |
| `--max <n>` | 50 | Maximum number of entries in output |
| `--max-depth <n>` | 5 | Maximum traversal depth |
| `--filter <regex>` | `.*` | Filter by full path (regex) |
| `--json` | off | Output as JSON array instead of TSV |

**Behavior:**
- Outputs flat list of instance paths with metadata
- Sorted by depth first, then alphabetically within each level
- Default output: TSV (no header, self-documenting format)
- If results exceed `--max`, prints warning to stderr

**Examples:**
```bash
# List all modules (max 50, TSV output)
wavepeek tree --waves dump.vcd

# Find ALU-related modules
wavepeek tree --waves dump.vcd --filter ".*alu.*"

# Explore 2 levels deep, as JSON
wavepeek tree --waves dump.vcd --max-depth 2 --json

# Get all modules
wavepeek tree --waves dump.vcd --max 1000
```

#### 3.2.3 `signals` — Signal listing

Lists signals within a specific scope with their metadata.

```
wavepeek signals --waves <file> --scope <path> [--max <n>] [--filter <regex>] [--json]
```

**Parameters:**
| Parameter | Default | Description |
|-----------|---------|-------------|
| `--waves <file>` | required | Path to VCD/FST file |
| `--scope <path>` | required | Exact scope path (e.g., `top.cpu`) |
| `--max <n>` | 50 | Maximum number of entries in output |
| `--filter <regex>` | `.*` | Filter by signal name (regex) |
| `--json` | off | Output as JSON array instead of TSV |

**Behavior:**
- Lists signals directly in the specified scope (non-recursive)
- Output fields: name, type, width, full path
- Sorted alphabetically by name
- Default output: TSV (no header, self-documenting format)
- If results exceed `--max`, prints warning to stderr

**Examples:**
```bash
# List signals in top.cpu
wavepeek signals --waves dump.vcd --scope top.cpu

# Find clock signals
wavepeek signals --waves dump.vcd --scope top.cpu --filter ".*clk.*"

# Get as JSON
wavepeek signals --waves dump.vcd --scope top.cpu --json
```

#### 3.2.4 `at` — Value extraction at time point

Gets signal values at a specific time point.

```
wavepeek at --waves <file> --time <time> [--scope <path>] --signals <names> [--json]
```

**Parameters:**
| Parameter | Default | Description |
|-----------|---------|-------------|
| `--waves <file>` | required | Path to VCD/FST file |
| `--time <time>` | required | Time point with units (e.g., `1337ns`, `10us`) |
| `--scope <path>` | — | Scope for short signal names |
| `--signals <names>` | required | Comma-separated signal names |
| `--json` | off | Output as JSON array instead of TSV |

**Modes:**
- `--signals clk,data` (no scope) — names are full paths
- `--scope top.cpu --signals clk,data` — names are relative to scope

**Behavior:**
- Outputs signal values at specified time
- Output fields: name (or path if no scope), value (hex)
- Values always in hex format
- Fail fast: error if any signal not found
- Default output: TSV (no header, self-documenting format)

**Examples:**
```bash
# Get values by full paths
wavepeek at --waves dump.vcd --time 100ns --signals top.cpu.clk,top.cpu.data

# Get values relative to scope (shorter)
wavepeek at --waves dump.vcd --time 100ns --scope top.cpu --signals clk,data,valid

# As JSON
wavepeek at --waves dump.vcd --time 100ns --scope top.cpu --signals clk --json
```

#### 3.2.5 `changes` — Value changes over time range

Outputs snapshots of signal values over a time range. Supports two modes:
unclocked (trigger on any signal change) and clocked (trigger on posedge of a clock).

```
wavepeek changes --waves <file> [--from <time>] [--to <time>] [--scope <path>] --signals <names> [--clk <name>] [--max <n>] [--json]
```

**Parameters:**
| Parameter | Default | Description |
|-----------|---------|-------------|
| `--waves <file>` | required | Path to VCD/FST file |
| `--from <time>` | start of dump | Start time with units (e.g., `1us`) |
| `--to <time>` | end of dump | End time with units (e.g., `2us`) |
| `--scope <path>` | — | Scope for short signal/clock names |
| `--signals <names>` | required | Comma-separated signal names |
| `--clk <name>` | — | Clock signal for posedge-triggered snapshots |
| `--max <n>` | 50 | Maximum number of snapshot rows |
| `--json` | off | Output as JSON array instead of TSV |

**Modes:**
- **Unclocked** (no `--clk`): row appears when any of `--signals` changes
- **Clocked** (`--clk` provided): row appears on each posedge of the clock signal. Clock itself is excluded from output.
- `--signals` and `--clk` follow same scope convention: short name with `--scope`, full path without

**Behavior:**
- Each row shows current value of all signals (snapshot)
- Output: `time\tname=value\tname=value\t...` (tab-separated)
- Signal order matches `--signals` order
- Values in hex format
- Fail fast: error if any signal not found
- If no changes/posedges in the time range, empty stdout + warning to stderr
- If results exceed `--max`, prints warning to stderr

**Examples:**
```bash
# Unclocked: track changes in time range
wavepeek changes --waves dump.vcd --from 1us --to 2us --signals top.cpu.clk,top.cpu.data

# Unclocked: with scope (shorter names)
wavepeek changes --waves dump.vcd --from 1us --to 2us --scope top.cpu --signals clk,data,valid

# Clocked: snapshot per clock cycle
wavepeek changes --waves dump.vcd --from 1us --to 2us --scope top.cpu --clk clk --signals data,valid,ready

# As JSON
wavepeek changes --waves dump.vcd --from 1us --to 2us --scope top.cpu --signals clk --json
```

#### 3.2.6 `when` — Event search

Finds clock cycles where a boolean expression evaluates to true.
Expression is evaluated on every posedge of the specified clock.

```
wavepeek when --waves <file> --clk <name> [--from <time>] [--to <time>] [--scope <path>] --cond <expr> [--first [<n>]] [--last [<n>]] [--max <n>] [--json]
```

**Parameters:**
| Parameter | Default | Description |
|-----------|---------|-------------|
| `--waves <file>` | required | Path to VCD/FST file |
| `--clk <name>` | required | Clock signal for posedge sampling |
| `--from <time>` | start of dump | Start of time range |
| `--to <time>` | end of dump | End of time range |
| `--scope <path>` | — | Scope for short signal/clock names in expression |
| `--cond <expr>` | required | Boolean expression in expression language |
| `--first [<n>]` | 1 if flag present | Return first N matches |
| `--last [<n>]` | 1 if flag present | Return last N matches |
| `--max <n>` | 50 | Maximum matches (when neither --first nor --last) |
| `--json` | off | Output as JSON array instead of TSV |

**Behavior:**
- Expression is evaluated at every posedge of `--clk`, outputs timestamps where it's true
- Output: one timestamp per line
- `--clk` follows same scope convention as `--signals`: short name with `--scope`, full path without
- **No qualifier:** return all matches up to `--max`
- **`--first`:** return first N matches (default N=1 if no value given)
- **`--last`:** return last N matches (default N=1 if no value given)
- `--first` and `--last` are mutually exclusive — error if both specified
- Error if `--first N` with `--max M` where N > M
- Fail fast: error if any signal in expression not found
- If no matches, empty stdout + warning to stderr

**Expression language (MVP):**

Operands:
- Signal names (scope-aware): `valid`, `data`, `top.cpu.data`
- Hex literal: `0xff`
- Binary literal: `0b1010`
- Decimal literal: `42`
- All values are unsigned

Operators (by precedence, highest first):

| Precedence | Operator | Description |
|------------|----------|-------------|
| 1 | `!` | Logical NOT |
| 2 | `<`, `>`, `<=`, `>=` | Comparison (unsigned) |
| 3 | `==`, `!=` | Equality |
| 4 | `&&` | Logical AND |
| 5 | `\|\|` | Logical OR |

Grouping: `(`, `)`

Truthy semantics: signal used without comparison operator is truthy if non-zero
(e.g., `valid` means `valid != 0`).

Post-MVP additions:
- Bit select/slice: `data[7:0]`, `data[3]`
- Bitwise operators: `&`, `|`, `^`, `~`
- Arithmetic: `+`, `-`
- Shift: `<<`, `>>`
- Signed comparison
- `x`/`z` value handling

**Examples:**
```bash
# Find all clock cycles when data equals 0xff (up to 50)
wavepeek when --waves dump.vcd --clk clk --cond "data == 0xff" --scope top.cpu

# First cycle where valid handshake occurs
wavepeek when --waves dump.vcd --clk clk --cond "valid && ready" --scope top.cpu --first

# Last 3 cycles where error was asserted
wavepeek when --waves dump.vcd --clk clk --cond "err == 1" --scope top.cpu --last 3

# Search within time window
wavepeek when --waves dump.vcd --clk top.cpu.clk --from 1us --to 2us --cond "top.cpu.data == 0xff"

# Complex expression
wavepeek when --waves dump.vcd --clk clk --scope top.cpu --cond "(a || b) && !reset"
```

---

## 4. Non-Functional Requirements

### 4.1 Performance
- Performance is the highest priority
- Rust is chosen specifically for this reason
- Benchmarks will be added as the project matures

### 4.2 Compatibility
- OS agnostic: Linux, macOS, Windows

### 4.3 Output Stability
- Deterministic output for identical input data

### 4.4 LLM Agent Integration
- Ready-made skill definition for LLM CLI agents (OpenCode, Codex CLI, Claude Code)
- Skill shipped in repo with setup instructions

---

## 5. Technical Architecture

### 5.1 Technology Stack

| Component | Choice | Rationale |
|-----------|--------|-----------|
| **Language** | Rust 1.93+ | Performance, memory safety, zero-cost abstractions. Ideal for parsing large binary/text dump files without GC pauses. |
| **CLI framework** | `clap` (derive API) | De-facto standard for Rust CLIs. Derive API provides self-documenting argument definitions with compile-time validation. |
| **Waveform parsing** | `wellen` | Unified interface for VCD, FST, and GHW formats. Battle-tested in the [Surfer](https://surfer-project.org/) waveform viewer. Multi-threaded VCD parsing via `rayon`. Optimized for on-demand signal access (loads hierarchy first, signal data lazily). |
| **Serialization** | `serde` + `serde_json` | Standard Rust serialization. Used for `--json` output in all commands. |
| **Pattern matching** | `regex` | For `--filter` flag in `tree` and `signals` commands. |
| **Error handling** | `thiserror` | Typed error enums with `#[derive(Error)]`. All error variants are known at compile time. No runtime error boxing (no `anyhow`). |
| **Build automation** | Cargo + Make | Cargo for compilation. Makefile provides shorthand targets: `make fmt`, `make lint`, `make test`, `make check`. |

### 5.2 High-Level Architecture

The system is organized in three layers. Data flows top-down: CLI parses arguments,
delegates to the engine, which reads waveform data and returns structured results.
The CLI layer formats results for output.

**Layers (top to bottom):**

1. **CLI Layer** (`clap`) — Argument parsing, validation, output formatting.
   TSV / JSON output to stdout, errors to stderr.
   Passes typed command structs down to the engine.

2. **Engine Layer** — Business logic per command: `info`, `tree`, `signals`,
   `at`, `changes`, `when`. Operates on waveform abstractions, returns structured
   results. Contains expression evaluator (for `when`).

3. **Waveform Layer** (`wellen`) — Thin adapter over wellen. Handles file opening,
   format detection (VCD/FST/GHW), hierarchy traversal, and signal value queries.
   Exposes: Hierarchy, Signal, TimeTable.

**Key architectural decisions:**

- **No intermediate data model.** We use `wellen::Hierarchy`, `wellen::Signal`, and
  `wellen::SignalValue` directly rather than converting to our own types.
  This avoids allocation overhead and keeps the codebase small.
  If wellen's API changes, the impact is contained to the waveform layer.

- **Stateless execution.** Each CLI invocation opens the file, executes one command,
  and exits. No caching, no sessions, no background processes. This simplifies the
  architecture and matches the design principle of minimal footprint.

- **Format-agnostic engine.** The engine layer does not know whether it is working
  with VCD, FST, or GHW. Format detection and loading is handled entirely by
  the waveform layer (wellen auto-detects format from file content).

### 5.3 Module Structure

Single crate with internal module boundaries. This is sufficient for a focused CLI tool
with 6 commands. If a library target is needed later (e.g., for an MCP server), the
engine and waveform modules can be extracted into a workspace crate with minimal effort.

```
src/
├── main.rs              # Entry point, top-level error handling
├── cli/                 # CLI layer: argument definitions and output formatting
│   ├── mod.rs           # Top-level CLI struct, subcommand dispatch
│   ├── info.rs          # `info` command args + output
│   ├── tree.rs          # `tree` command args + output
│   ├── signals.rs       # `signals` command args + output
│   ├── at.rs            # `at` command args + output
│   ├── changes.rs       # `changes` command args + output
│   └── when.rs          # `when` command args + output
├── engine/              # Business logic per command
│   ├── mod.rs           # Shared types (result structs, time parsing)
│   ├── info.rs          # Dump metadata extraction
│   ├── tree.rs          # Hierarchy traversal with depth/filter
│   ├── signals.rs       # Signal listing within scope
│   ├── at.rs            # Value extraction at time point
│   ├── changes.rs       # Value change tracking (clocked/unclocked)
│   └── when.rs          # Condition evaluation over clock cycles
├── expr/                # Expression engine (for `when` command)
│   ├── mod.rs           # Public API: parse() + eval()
│   ├── lexer.rs         # Tokenizer
│   ├── parser.rs        # Expression parser → AST
│   └── eval.rs          # AST evaluator against signal values
├── waveform/            # Thin adapter over wellen
│   └── mod.rs           # File loading, format detection, query helpers
├── output.rs            # Shared output formatting (TSV, JSON)
└── error.rs             # Error enum (WavepeekError)
```

**Separation of concerns:**

| Module | Knows about | Does not know about |
|--------|-------------|---------------------|
| `cli/` | clap, output formats (TSV/JSON) | wellen, file I/O |
| `engine/` | domain logic, waveform layer API | clap, output formats |
| `expr/` | expression AST, signal values | wellen, CLI, output |
| `waveform/` | wellen API | CLI, engine logic |
| `output` | serde_json, TSV formatting | domain logic |
| `error` | all error variants | nothing else |

### 5.4 Key Dependencies

| Crate | Version | Purpose | Notes |
|-------|---------|---------|-------|
| `wellen` | ~0.20 | VCD, FST, GHW parsing | Core dependency. BSD-3-Clause. Brings in `rayon`, `fst-reader`, `memmap2`, `lz4_flex` transitively. |
| `clap` | ~4 | CLI argument parsing | With `derive` feature for declarative argument definitions. |
| `serde` | ~1 | Serialization framework | With `derive` feature. |
| `serde_json` | ~1 | JSON output | For `--json` flag. |
| `regex` | ~1 | Pattern matching | For `--filter` in `tree` and `signals`. |
| `thiserror` | ~2 | Error type derivation | `#[derive(Error)]` for typed error enums. |

**Dev dependencies:**

| Crate | Purpose |
|-------|---------|
| `assert_cmd` | CLI integration testing (run binary, assert stdout/stderr/exit code) |
| `predicates` | Assertion helpers for `assert_cmd` |
| `tempfile` | Temporary file creation for tests |

### 5.5 Expression Engine

The `when` command requires evaluating boolean expressions against signal values
at each clock posedge. This needs a small expression language (defined in 3.2.6).

**Pipeline:** Input string → Lexer → Token stream → Parser → AST → Evaluator (+ signal values at current time point) → bool

**Components:**

- **Lexer** — Converts input string into tokens: identifiers (signal names),
  literals (hex `0xff`, binary `0b1010`, decimal `42`), operators (`==`, `!=`,
  `&&`, `||`, `!`, `<`, `>`, `<=`, `>=`), and parentheses.

- **Parser** — Pratt parser or recursive descent. Produces an AST respecting
  operator precedence (see 3.2.6). All signal names in the AST are strings
  that get resolved against the waveform hierarchy at evaluation time.

- **AST** — Enum-based tree: `BinaryOp(op, lhs, rhs)`, `UnaryOp(op, expr)`,
  `Signal(name)`, `Literal(value)`. Minimal, no optimization passes.

- **Evaluator** — Walks the AST, resolves signal names to current values
  (provided by the engine), computes the result. All arithmetic is unsigned.
  Truthy semantics: any non-zero value is true.

**Implementation approach:** Deferred. The expression language is small enough
(5 precedence levels, no user-defined functions) that both hand-written recursive
descent and parser combinators (`nom`, `winnow`) are viable.
The decision will be made during implementation based on error message quality
requirements and complexity of the post-MVP extensions (bit slicing, bitwise ops).

### 5.6 Error Handling Strategy

**Principles:**
- **Fail fast.** On the first error, print a message to stderr and exit with a
  non-zero code. No partial output to stdout.
- **LLM-parseable errors.** Error messages follow a fixed format so agents can
  detect and interpret them programmatically.
- **No panics in production paths.** All recoverable errors use `Result<T, WavepeekError>`.
  `unwrap()` / `expect()` only in cases that indicate programmer bugs.

**Error format (stderr):**

```
error: <category>: <message>
```

Examples:
```
error: file: cannot open 'dump.vcd': No such file or directory
error: args: --time requires units (e.g., '100ns'), got '100'
error: signal: signal 'top.cpu.foo' not found in dump
error: expr: parse error in condition: unexpected token ')' at position 12
```

**Exit codes:**

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | User error (bad arguments, signal not found, invalid expression) |
| `2` | File error (cannot open, cannot parse, unsupported format) |

**Error enum:**

A single `WavepeekError` enum in `src/error.rs` with variants covering all failure
modes. Each variant maps to an exit code and a formatted message.
The CLI layer converts `WavepeekError` into stderr output and exit code.

### 5.7 Testing Strategy

**Levels:**

| Level | What | How | Fixtures |
|-------|------|-----|----------|
| **Unit tests** | Individual functions in `engine/`, `expr/`, `waveform/` | `#[cfg(test)]` modules, `cargo test` | Hand-crafted VCD strings (inline or small `.vcd` files) |
| **Integration tests** | Full CLI invocations end-to-end | `assert_cmd` in `tests/` directory | Verilator-generated VCD/FST files |
| **Expression tests** | Lexer, parser, evaluator independently | Unit tests in `expr/` submodules | None (pure logic, string inputs) |

**Test fixture strategy (two sources):**

1. **Hand-crafted VCD files** — For unit tests and edge cases. VCD is a simple
   text format, easy to write manually. Covers: empty files, single-bit signals,
   multi-bit signals, X/Z values, multiple scopes, deep hierarchy, time edge cases
   (zero duration, single timestamp). Stored in `tests/fixtures/hand/`.

2. **Verilator-generated fixtures** — For integration tests. Small Verilog modules
   simulated with Verilator to produce realistic VCD/FST dumps. Covers: realistic
   hierarchy names, typical signal patterns, clock-data relationships, multi-format
   consistency (same design exported as VCD and FST should produce identical output).
   Source Verilog in `tests/fixtures/src/`, generated dumps in `tests/fixtures/gen/`.
   Generation is a manual step (not part of `cargo test`), with a Makefile target
   `make fixtures`.

**What to assert in integration tests:**

- Exact stdout output (deterministic output is a design principle)
- Exit code
- Stderr content for error cases
- `--json` output validates against expected JSON structure
- Consistency: same query on VCD and FST of the same design produces identical output

---

## 6. Roadmap

### Phase 1: MVP (Core Functionality)
<!-- TODO: Define MVP scope -->
- [ ] Basic VCD parsing
- [ ] Signal listing and search
- [ ] Value extraction at time point
- [ ] Text output

### Phase 2: Extended Features
- [ ] FST format support
- [ ] JSON output
- [ ] Conditional queries
- [ ] Range queries

### Phase 3: Advanced
- [ ] Performance optimization
- [ ] Additional formats (EVCD, GHW)
- [ ] Multi-signal correlation
- [ ] Plugin system

---

## 7. Open Questions
<!-- TODO: Resolve open questions -->
1. What syntax for conditional queries? (SQL-like? Expression-based?)
2. Is streaming mode needed for huge files?
3. Priority of formats after VCD?
4. Is an interactive REPL session needed?
5. Integration with other tools (Verilator, cocotb)?

---

## 8. References
- [GTKWave](http://gtkwave.sourceforge.net/) — reference waveform viewer
- [VCD Format Specification](https://en.wikipedia.org/wiki/Value_change_dump)
- [FST Format](https://gtkwave.sourceforge.net/gtkwave.pdf)


