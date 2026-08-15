# Keep fatal output machine-readable

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

A caller that selects `--json` or `--jsonl` must always be able to decode a handled command result, including fatal failures. After this change, argument, file, query, expression, mapping, and runtime failures use a flat fatal JSON object instead of a human line on stderr. JSONL failures append a terminal `fatal` record at the next stream sequence number. Human output, successful output, diagnostics, exit statuses, debug tracing, and broken-pipe handling remain unchanged.

The behavior is visible by running `wavepeek --json info --waves missing.vcd`: stdout contains one object with `type: "fatal"`, code `WPK-F0002`, and the file message, while stderr is empty and the process exits with status 2.

## Non-Goals

This work does not change ordinary diagnostic objects, success envelopes, stream data, waveform execution, error categories, exit statuses, help/version rendering, or broken-pipe semantics. It does not make the human-only `skill` command support machine results; asking for a machine mode still fails, but that failure uses the requested machine format. It adds no dependencies or speculative output abstraction.

## Progress

- [x] (2026-08-15 05:16Z) Read issue #104, repository guidance, execution path, output contracts, and existing tests.
- [x] (2026-08-15 05:17Z) Confirm selector boundary semantics with the maintainer and start the worktree devcontainer with hooks installed.
- [x] (2026-08-15 05:26Z) Added focused tests for selector placement, fatal shapes and codes, stream sequencing, stderr separation, help/version, and selector boundaries.
- [x] (2026-08-15 05:26Z) Implemented early output-mode selection and centralized fatal serialization in the CLI and existing output writer.
- [x] (2026-08-15 05:27Z) Updated normative machine-output and command-model references plus maintainer style and architecture wording.
- [x] (2026-08-15 05:32Z) Ran focused tests and `./dev just ci`; all gates pass. Implementation commit remains.
- [ ] Run two focused review waves and one independent control review (completed: Luna Max correctness, architecture, and contract lanes; fixed all findings; remaining: rerun CI, commit, Terra High wave, Sol High control).
- [ ] Remove this branch-local plan, commit cleanup, push, and open the pull request.

## Surprises & Discoveries

- Observation: JSONL streaming writers live in `src/cli/mod.rs` across the full `engine::run_jsonl` call, so a runtime failure can reuse the writer's private next sequence number instead of introducing engine-wide error plumbing.
  Evidence: `dispatch` creates `JsonlWriter`, and streaming sinks call `begin`, `data`, and `end` on that same writer.

- Observation: non-streaming JSONL commands can fail before `begin` because execution completes before `output::write_jsonl_result` starts writing.
  Evidence: `engine::run_jsonl` delegates `info`, `scope`, `signal`, and `value` to `run(command)?` before rendering.

- Observation: `cargo test --lib` alone assumes generated waveform fixtures already exist, while `just ci` prepares them before the coverage suite.
  Evidence: the direct library run had 665 passing tests and two missing-fixture failures; `just ci` generated the fixtures and passed all source, docs, coverage, and FSDB gates.

- Observation: looking up a preceding value-taking option across the entire clap tree suppresses selectors after unrelated options.
  Evidence: the first review wave found that `info --profile --json` treated `--json` as a value because `--profile` exists on another subcommand. Restricting lookup to the argv-selected command path restores JSON fatal output.

- Observation: an existing DEBUG tuning path provides a real post-`begin` JSONL failure without test-only hooks.
  Evidence: forced streaming candidate collection on a VCD emits `begin` at sequence 0 and an internal fatal at sequence 1, with no `end`.

## Decision Log

- Decision: Treat exact `--json` and `--jsonl` tokens as selectors only when clap would treat them as options; never inspect text after `--` or values such as `--eval=--json`.
  Rationale: The maintainer explicitly rejected interpreting selector-like value text as an output mode. This preserves the conventional option terminator and avoids corrupting expression or path values.
  Date/Author: 2026-08-15 / pi

- Decision: Ignore machine selectors for successful help and version requests.
  Rationale: The maintainer confirmed that help and version remain human-readable success surfaces.
  Date/Author: 2026-08-15 / pi

- Decision: Test production-unreachable fatal variants at the nearest direct serialization boundary.
  Rationale: Artificial CLI-only failure hooks would add production complexity solely for tests.
  Date/Author: 2026-08-15 / pi

- Decision: Keep fatal code and raw message mapping on `WavepeekError`, and keep stdout rendering in `src/output.rs`.
  Rationale: `src/error.rs` already owns categories and exit codes, while `src/output.rs` owns all machine serialization. Reusing those boundaries is smaller than a new module or error hierarchy.
  Date/Author: 2026-08-15 / pi

- Decision: Preserve the public `run_cli() -> Result<(), WavepeekError>` API and add a hidden process-status wrapper for the binary.
  Rationale: Luna Max identified the initial `ExitCode` return as an unnecessary downstream API break. The result API can remain while the binary alone suppresses already-reported machine errors.
  Date/Author: 2026-08-15 / pi

## Outcomes & Retrospective

The implementation now emits one decodable terminal result for handled machine-mode failures without human fatal duplication on stderr. Focused tests cover pre-parse argument errors, pre-begin file and signal failures, JSONL sequencing, selector conflict behavior, selector boundaries, helper rejection, help/version, debug telemetry, and human compatibility. The full CI gate passes; required peer review, cleanup, and PR creation remain.

## Context and Orientation

`src/main.rs` is the process wrapper. It calls `wavepeek::run_cli()` from `src/lib.rs`, prints every returned non-broken-pipe error to stderr, and maps `WavepeekError` to an exit status.

`src/cli/mod.rs` defines the clap command tree, parses `std::env::args_os`, normalizes clap failures, converts parsed commands into `src/engine/mod.rs::Command`, and dispatches execution. Today each waveform command owns local `json` and `jsonl` booleans. A parse error happens before those booleans can be read, which is the root cause of human fatal output for malformed machine invocations.

`src/error.rs` defines `WavepeekError`. Its `Display` implementation, generated by `thiserror`, includes the human `fatal: <category>:` prefix. Its `exit_code` method preserves status 2 for file failures, status 1 for other handled failures, and status 0 for broken pipes.

`src/output.rs` renders human and JSON results and defines `JsonlWriter`. The writer owns `next_seq`; every successful record increments it. A new fatal operation here can therefore append a fatal record after any already-written stream records without exposing sequence state elsewhere.

`src/contract/stream.rs` contains JSONL transfer objects. A fatal record differs from begin/data/diagnostic/end because it deliberately has no command field. JSON fatal output uses the same flat fields without `seq`.

Integration tests under `tests/` invoke the compiled CLI using `tests/common/mod.rs::wavepeek_cmd`. Existing suites cover clap failures, JSONL sequence ordering, streaming runtime behavior, stderr, debug tracing, and broken pipes. A focused `tests/fatal_output_cli.rs` suite can cover the new cross-cutting contract without expanding command fixture manifests.

The normative user contract is `skills/wavepeek/references/machine-output.md`, with cross-cutting mode placement in `skills/wavepeek/references/command-model.md`. `docs/style.md` currently says all errors go to stderr and must be narrowed to human mode.

A fatal code is a stable string identifying a `WavepeekError` category: arguments `WPK-F0001`, file `WPK-F0002`, scope `WPK-F0003`, signal and signal-not-found `WPK-F0004`, expression `WPK-F0005`, internal `WPK-F0006`, and unimplemented `WPK-F0007`.

## Open Questions

None. The maintainer resolved selector boundaries, help/version behavior, and unreachable-category test depth.

## Plan of Work

First add focused tests that exercise the public contract. Cover `--json` and `--jsonl` before and after a complete command path, malformed arguments and unknown subcommands before dispatch, missing files and query failures before a JSONL begin, a streaming expression/runtime failure after begin, stable category mappings, conflicting machine flags, empty stderr outside debug mode, retained debug telemetry, human failure rendering, and successful help/version output despite machine flags. Add direct unit tests where internal and unimplemented variants cannot be reached from the public CLI. Include value-boundary cases proving selector-like text after `--` or inside `--option=value` is not selected.

Then change `src/output_mode.rs` and `src/cli/mod.rs` so the requested mode is known before full clap validation and machine selectors can appear at any option position before `--`. Keep successful help/version on their existing output path. Ensure both selectors deterministically choose JSONL for the conflict fatal. Do not duplicate mode decisions in command engines.

Add raw fatal code/message accessors to `WavepeekError` in `src/error.rs`. Add the smallest serializable fatal records to the existing contract/output modules. Give `JsonlWriter` a fatal method that uses its current `next_seq`; add a standalone JSONL fatal writer for failures before a command name or writer exists. Centralize dispatch failure handling in `src/cli/mod.rs`: human failures still return to `main`, JSON failures write one object, and JSONL failures write or append one terminal record. If writing that fatal itself fails, return the write/serialization error so `main` retains the existing broken-pipe and fallback behavior.

Update the normative documents to describe global selector placement, fatal JSON/JSONL shapes, codes, stream termination, stderr separation, and exceptions. Keep prose concise and avoid duplicating exact command-local flag listings.

Run focused tests while iterating, then the full CI gate. Commit the implementation using a conventional commit that closes #104. Launch read-only review lanes for correctness/tests, architecture/complexity, and docs/contracts. Wave one uses Luna Max; wave two repeats the same lanes with Terra High after fixes. Every prompt explicitly applies KISS, YAGNI, and ponytail-review. Apply concrete findings only, rerun affected tests, and finish with a fresh Sol High control review of the consolidated diff. Remove this `docs/wip/` plan before the final pull request unless a maintainer requests it remain.

### Concrete Steps

Work from repository root `/home/esynr3z/projects/wavepeek/.worktrees/wavepeek/104`.

Start and prepare the worktree container once:

    ./dev --install-hooks
    ./dev

Run focused tests during implementation:

    ./dev cargo test --test fatal_output_cli
    ./dev cargo test output_mode
    ./dev cargo test output::tests
    ./dev cargo test error::tests

Format and run the complete behavior gate:

    ./dev just format
    ./dev just ci

Inspect and commit:

    git status --short
    git diff --check
    git diff --stat
    git add <changed files>
    git commit -m "fix(cli): serialize fatal machine output" -m "Closes #104"

After review fixes and a clean control pass, remove the branch-local plan, rerun `./dev just check`, commit any final tracked changes, push the branch, and create the PR with `gh pr create` against the repository default development branch.

### Validation and Acceptance

A JSON file failure must produce exactly one stdout object equivalent to:

    {"type":"fatal","code":"WPK-F0002","message":"..."}

It must have empty stderr with `DEBUG` unset and exit status 2. A malformed JSON invocation must use `WPK-F0001` and status 1.

A JSONL failure before begin must produce one line with `type: "fatal"`, `seq: 0`, no `command`, empty stderr, and the existing nonzero status. A streaming failure after begin must preserve prior records and append fatal with the next contiguous sequence number; no `end` may follow it.

When both selectors are present, stdout must contain one JSONL fatal at sequence zero. The line must also parse as a standalone JSON value.

Human failures must retain `fatal: <category>: <message>` on stderr and empty stdout. `DEBUG=1` may retain telemetry JSON on stderr. Help and version must remain successful human text even if a machine selector is present. Selector-looking argument values and tokens after `--` must not change the chosen mode.

Successful JSON must still emit exactly one result object. Successful JSONL must still start with one begin and end with one end. Existing ordinary diagnostics and broken-pipe tests must continue passing.

The final `./dev just ci` command must exit zero. After the final review fixes, `./dev just check` must exit zero with hooks active.

### Idempotence and Recovery

All test and formatting commands are safe to rerun. The container and hook installation commands are idempotent. If a fatal write fails because stdout is closed, preserve the existing error path rather than retrying output. If a review fix regresses behavior, revert only that focused diff and rerun the affected suite. Do not delete unrelated files under `tmp/`.

### Artifacts and Notes

Issue #104 requires these stable mappings:

    Args -> WPK-F0001
    File -> WPK-F0002
    Scope -> WPK-F0003
    Signal | SignalNotFound -> WPK-F0004
    Expr -> WPK-F0005
    Internal -> WPK-F0006
    Unimplemented -> WPK-F0007

The fatal message is the inner variant text, not `WavepeekError::to_string()`, because `to_string()` intentionally includes the human prefix.

### Interfaces and Dependencies

Use only existing `clap`, `serde`, and `serde_json` dependencies.

`WavepeekError` must expose crate-visible or public constant accessors equivalent to:

    pub const fn fatal_code(&self) -> Option<&'static str>
    pub fn message(&self) -> Option<&str>

Broken pipe is not a serializable fatal category, so these accessors may return `None` for `BrokenPipe`.

`JsonlWriter<W>` must expose a fatal operation that serializes the provided `WavepeekError` with the writer's current sequence number. A JSON fatal writer must serialize the same error fields without sequence. Keep record fields private unless existing contract tests require wider visibility.

Plan revision note (2026-08-15 05:16Z): Created the initial self-contained plan after repository exploration and maintainer decisions. The plan chooses existing error and output ownership boundaries to minimize new code.

Plan revision note (2026-08-15 05:32Z): Recorded completed implementation, documentation, focused tests, the passing full CI gate, and the generated-fixture behavior discovered during validation.

Plan revision note (2026-08-15 06:05Z): Recorded the Luna Max review wave and fixes: active command-path selector scanning, public API preservation, real post-begin integration coverage, removal of a duplicate output-mode assignment, and architecture wording correction.
