# Expose result completeness and summary-only output

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

WavePeek currently reports count-limit truncation only through diagnostic `WPK-W0002`, and its JSONL record count describes emitted records rather than query completeness. After this work, every successful command that accepts `--max` will expose one four-field summary describing whether the selected result set was fully scanned, how many public items were accepted, the active numeric limit, and the exact total when already known. Users can also pass `--summary` to run the same bounded query while suppressing result rows, retaining command context and diagnostics.

The behavior is visible by running a bounded command in JSON or JSONL mode. JSON will contain top-level `summary`; JSONL will contain the identical object in its terminal `end` record. With `--summary`, JSON omits `data`, while JSONL emits no `data` records and reports `records.data: 0`; `summary.returned` still describes the query result before rendering suppression.

## Non-Goals

This work does not add or change fatal JSONL records. It does not make a second scan to compute an exact total. It does not treat `--max-depth` as truncation because depth is a selector. It does not add `--summary` to commands without `--max`. It does not redesign command context: existing protocol context and context added independently by issue #114 must pass through unchanged. It does not add dependencies or speculative output abstractions.

## Progress

- [x] (2026-08-14 14:24Z) Read issue #92, related output-contract issue #106, context issue #114, architecture, testing, style, quality, and local breadcrumb guidance.
- [x] (2026-08-14 14:24Z) Install worktree hooks and start the devcontainer.
- [x] (2026-08-14 14:45Z) Add the shared summary state and `--summary` CLI surface to every command with `--max`.
- [x] (2026-08-14 14:45Z) Preserve existing query execution while propagating summary values through list, row, event, and streaming paths.
- [x] (2026-08-14 14:45Z) Render summaries consistently in human summary-only, JSON, and JSONL modes while preserving optional context and diagnostics.
- [x] (2026-08-14 14:45Z) Add focused runtime and contract coverage for complete, exactly-at-limit, truncated, unlimited, max-depth, and summary-only cases.
- [x] (2026-08-14 14:51Z) Update packaged machine-output, command-model, command references, and bundled skill guidance.
- [ ] Run focused tests, `./dev just ci`, and `./dev just check`; record evidence (focused Rust, clippy, and docs checks pass; full gates remain).
- [x] (2026-08-14 14:51Z) Commit implementation milestones with conventional commits.
- [ ] Run Luna Max focused review wave, fix findings, then Terra High focused review wave over the same lanes and fix findings.
- [ ] Run independent Sol High control review, resolve findings, rerun gates, remove this WIP plan, and commit cleanup.
- [ ] Push the branch and open a pull request against `dev3`.

## Surprises & Discoveries

- Observation: list commands already collect the full selected set before truncating, while row and event engines already stop only after finding one additional accepted public item.
  Evidence: `src/engine/scope.rs` and `src/engine/signal.rs` compare the full length with the numeric limit; `ChangeRunStats`, `PropertyRunStats`, `ExtractRunStats`, and the AHB walker already track `emitted` and `truncated`.

- Observation: issue #92 contains an older-looking JSONL `end` example that repeats `command`, but the implemented v3 contract from issue #106 requires `command` only in `begin` and retains `end.records`.
  Evidence: `src/contract/stream.rs::EndRecord`, `skills/wavepeek/references/machine-output.md`, and issue #106 all use `end.records`; issue #106 explicitly maps JSON `summary` to JSONL `end.summary`.

- Observation: `./dev --start` is not a supported no-command option, but invoking it started the worktree container before reporting an argument execution error.
  Evidence: devcontainer CLI reported container `c64fe2ce6d99f4a2631cfe67d3332d257310d8c0e0dc343f5a121ec58794ec41` started, followed by `No such file or directory: '--start'`; subsequent work should use `./dev --exec-only` or ordinary `./dev just ...` commands.

## Decision Log

- Decision: Keep JSONL `command` and optional `context` in `begin`, keep `records` in `end`, and add `summary` beside `records`.
  Rationale: This preserves the normalized v3 stream contract from issue #106. `records.data` counts physically emitted records, while `summary.returned` counts accepted query results, so summary-only streams can truthfully report zero emitted rows and a nonzero returned count.
  Date/Author: 2026-08-14 / pi

- Decision: In summary-only mode, omit JSON `data`, emit no JSONL `data` records, and preserve optional context and all diagnostics.
  Rationale: This follows the current issue #92 wording and the user's explicit interpretation that only result data is suppressed. Context is command-wide metadata and diagnostics remain observable.
  Date/Author: 2026-08-14 / pi

- Decision: Represent completeness once in a small shared value carried by `CommandResult`, and reuse existing engine limit state instead of rescanning or deriving completeness in renderers.
  Rationale: One source of truth prevents JSON/JSONL drift and is the smallest design compatible with collected and streaming paths.
  Date/Author: 2026-08-14 / pi

- Decision: Preserve context opaquely rather than adding summary-specific context variants or coupling.
  Rationale: Issue #114 independently adds scope context to `signal`, scoped `value`, scoped `change`, and scoped `extract generic`. Summary rendering only needs to refrain from dropping whichever context exists.
  Date/Author: 2026-08-14 / pi

## Outcomes & Retrospective

The implementation milestone now exposes summaries across all ten bounded commands without changing their existing limit walkers. A dedicated integration suite covers list, row, and event producers plus summary-only context and diagnostic retention; the full Rust test suite passes with 664 unit tests and all integration binaries, and both default and FSDB clippy passes are clean. Documentation, full repository gates, review waves, and PR creation remain.

## Context and Orientation

WavePeek is a Rust command-line tool. `src/cli/` defines clap argument structures. `src/engine/` opens waveform files, applies selectors and filters, enforces limits, and returns `CommandResult` for collected output or writes rows through `JsonlWriter` for streaming output. `src/contract/output.rs` converts collected results into JSON contract data transfer objects. `src/contract/stream.rs` defines JSONL records. `src/output.rs` renders human output, JSON envelopes, and JSONL streams.

The affected command paths are `scope`, `signal`, `change`, `property`, and `extract ahb`, `extract apb`, `extract atb`, `extract axi`, `extract axistream`, and `extract generic`. These are exactly the commands that accept `--max`. `src/cli/limits.rs::LimitArg` represents either `Numeric(usize)` or `Unlimited`; `LimitArg::numeric()` already maps directly to the nullable summary limit.

A public item means one scope entry, signal entry, change row, property capture row, or extraction event/transfer/row after all command selection and filtering rules. `complete` is false only after execution observes another matching public item beyond a numeric limit. `returned` is the number accepted before that limit. `total` equals `returned` when execution reaches the selected result-set end. For list commands, the engine already has the full selected count before truncation, so an exact total can remain known even when output is incomplete. For early-stopping row and event engines, a truncated result has unknown total and must use `null`.

`--max-depth` is a selector: it defines which hierarchy entries belong to the selected set. Excluded deeper entries do not make that selected set incomplete. `--max unlimited` scans the complete selected set and therefore yields `complete: true`, `limit: null`, and `total == returned`.

JSONL is a sequence rather than one object. `begin` contains the command and optional command-wide context. Each public row normally becomes a `data` record. Diagnostics become `diagnostic` records. A successful stream ends with `end`, whose `records` object counts emitted data and diagnostic records. This work adds the same `summary` object used by JSON to `end`. Under `--summary`, no data records are emitted, so `records.data` is zero even when `summary.returned` is nonzero.

Canonical user documentation is packaged under `skills/wavepeek/`. `skills/wavepeek/references/machine-output.md` defines JSON and JSONL behavior, while `skills/wavepeek/references/command-model.md` defines bounded-output semantics. Command-specific references explain the affected command families, and `skills/wavepeek/SKILL.md` gives concise agent guidance. Integration tests under `tests/` invoke the compiled CLI and inspect exact JSON, JSONL, stdout, stderr, and help behavior.

## Open Questions

There are no blocking product questions. If issue #114 lands before this branch is finalized, rebase onto its commit before the final review waves and add or update tests proving that summary-only JSON retains `context.scope` and summary-only JSONL retains it in `begin`. If it has not landed, keep the summary implementation context-agnostic and document the expected composition in the pull request.

## Plan of Work

First add a minimal shared `ResultSummary` structure near `CommandResult` in `src/engine/mod.rs`. It must contain `complete: bool`, `returned: usize`, `limit: Option<usize>`, and `total: Option<usize>`, derive serialization as needed, and be created from already-known execution state. Add a `summary_only` rendering flag to `CommandResult`; do not place rendering policy in every contract DTO.

Add `--summary` as a boolean output flag to each affected clap argument structure in `src/cli/scope.rs`, `src/cli/signal.rs`, `src/cli/change.rs`, `src/cli/property.rs`, and all six argument structures in `src/cli/extract.rs`. Do not expose it on `info`, `value`, `skill`, or any other command without `--max`.

In `src/engine/scope.rs` and `src/engine/signal.rs`, record the selected length before truncation. A numeric limit shorter than that length yields `complete: false`, `returned: limit`, and the already-known exact `total`. Otherwise execution is complete and `returned` and `total` equal the selected length. `--max-depth` remains only part of selection.

In `src/engine/change.rs`, `src/engine/property.rs`, and `src/engine/extract.rs`, convert existing emitted/truncated run statistics directly into summaries. A non-truncated run has exact `total == emitted`; a truncated early-stopped run has `total: None`. Propagate shared extraction statistics through APB, ATB, AXI, and AXI-Stream adapters rather than recomputing them. In `src/engine/ahb.rs`, preserve its existing one-extra-event truncation detection and expose the equivalent summary. Both collected and direct JSONL execution paths must receive the same summary.

In `src/contract/output.rs`, add top-level `summary` only for affected successful commands and make `data` conditionally omitted only when `summary_only` is true. Context construction must remain independent so protocol context and future scope context survive row suppression. In `src/contract/stream.rs`, extend `EndRecord` with the summary beside existing `records`; do not repeat `command` or context in `end`.

In `src/output.rs`, render human summary fields after any existing command-wide context while suppressing public result rows. Continue emitting human diagnostics to stderr. For JSONL, suppress calls that serialize data records while still executing the same sink path and counting accepted query rows in engine statistics. Preserve `records.data` as the number of serialized data records and pass the engine summary to `JsonlWriter::end`. Avoid buffering streaming commands or rescanning.

Add direct tests using existing fixtures. Shared parity tests should assert identical summary objects for JSON and terminal JSONL across every affected command. Focused list, row, and event tests must cover fewer-than-limit, exactly-at-limit, truncated, unlimited, and summary-only cases. Include a depth-selector test proving `--max-depth` does not make completeness false. Assert JSON omits `data` in summary-only mode, JSONL emits no `data` records, context remains where present, diagnostics remain, and `records.data` differs appropriately from `summary.returned`. Extend CLI help tests to ensure `--summary` appears only on commands with `--max`.

Update `skills/wavepeek/references/machine-output.md` with normative JSON and JSONL examples, including the distinction between emitted record counts and returned query items. Update `skills/wavepeek/references/command-model.md` with completeness semantics. Add concise summary usage to affected command-family references and `skills/wavepeek/SKILL.md`, avoiding duplicated exhaustive flag documentation.

After focused tests pass, run `./dev just ci` and `./dev just check`. Commit logical milestones. Then run three parallel Luna Max reviewer lanes covering correctness/tests, output contract/docs/context composition, and architecture/performance/KISS. Fix findings and rerun impacted tests. Repeat the same lanes with Terra High. Finally run one independent Sol High control review over the consolidated branch, fix any substantive findings, rerun full gates, remove this WIP plan, commit cleanup, push, and open a PR against `dev3`.

### Concrete Steps

Run all repository commands from `/home/esynr3z/projects/wavepeek/.worktrees/wavepeek/dev3-92`. The devcontainer is already started and hooks are installed.

Inspect and edit using repository tools, then run focused tests through the container, for example:

    ./dev cargo test --test json_jsonl_parity
    ./dev cargo test --test jsonl_cli
    ./dev cargo test --test scope_cli
    ./dev cargo test --test change_cli
    ./dev cargo test --test extract_generic_cli
    ./dev cargo test --test cli_contract

The expected result for each focused invocation is exit code zero with all selected tests passing. Run formatting before each implementation commit:

    ./dev just format

At the pre-review milestone run:

    ./dev just ci
    ./dev just check

Both commands must exit zero. The final commit hooks must also exit zero without bypassing verification.

After all review waves and cleanup, push and open the PR:

    git push -u origin dev3-92/result-completeness-summary
    gh pr create --repo kleverhq/wavepeek --base dev3 --head dev3-92/result-completeness-summary --title "feat(output): expose result completeness summaries" --body-file tmp/issue-92-pr.md

Confirm the PR URL is printed and record it in `Outcomes & Retrospective` before removing the plan, or in the final user response if the plan has already been removed.

### Validation and Acceptance

A complete bounded result below or exactly at its numeric limit must report `complete: true`, `returned` equal to the number of public items, `limit` equal to the numeric argument, and `total == returned`. Finding one additional matching public item must report `complete: false`, retain `WPK-W0002`, and use an exact total only if the command already knows it without another scan. Unlimited execution must report `limit: null`, `complete: true`, and an exact total.

Every successful JSON result for an affected command must include the four summary fields with stable names and types. Every successful JSONL stream for the same invocation must end with an identical summary. Commands without `--max` must not expose `--summary` or count-limit summary metadata.

Under `--summary`, the query must produce the same summary and diagnostics as the equivalent ordinary invocation. JSON must omit `data`; JSONL must emit no `data` records, keep `begin` and optional context, emit diagnostics, and finish with `end`. Human output must suppress result rows, preserve command-wide context when the command already renders it, print the four summary fields, and retain diagnostics on stderr.

The test suite must demonstrate list, row, and event producers; complete, exact-limit, truncated, unlimited, and summary-only results; selector-only max depth; JSON/JSONL parity; context retention; and stable help/docs. `./dev just ci` and `./dev just check` must pass after all review fixes.

### Idempotence and Recovery

All test, format, and quality commands are safe to rerun. The engine must not mutate waveform inputs. If a commit hook fails, keep the container running, fix the reported cause, rerun the focused command, and retry the commit without bypassing hooks. If issue #114 lands during work, commit or stash a clean local state, rebase onto updated `origin/dev3`, resolve only the shared output-contract files, and rerun context and summary parity tests. Do not delete unrelated files under `tmp/`.

### Artifacts and Notes

The intended summary-only JSONL shape is:

    {"type":"begin","seq":0,"command":"change","context":{"scope":"top.cpu"}}
    {"type":"diagnostic","seq":1,"diagnostic":{"kind":"warning","code":"WPK-W0002","message":"truncated output to 1 entries (use --max to increase limit)"}}
    {"type":"end","seq":2,"records":{"data":0,"diagnostics":1},"summary":{"complete":false,"returned":1,"limit":1,"total":null}}

The corresponding JSON shape contains `type`, `command`, optional `context`, `summary`, and `diagnostics`, with `data` omitted. The query summary is identical even though the JSONL emitted-record count is zero.

### Interfaces and Dependencies

Use existing `serde`, `serde_json`, clap, diagnostics, command result, and sink infrastructure. Add no dependency.

Define one shared summary DTO or engine value with the wire fields:

    pub struct ResultSummary {
        pub complete: bool,
        pub returned: usize,
        pub limit: Option<usize>,
        pub total: Option<usize>,
    }

`CommandResult` must carry `Option<ResultSummary>` because commands without `--max` have no summary, plus one rendering-only boolean controlling row suppression. JSON contracts borrow or serialize this value directly rather than defining command-specific copies. JSONL `EndRecord` accepts the summary for affected commands while retaining `RecordCounts`. Streaming engine functions must pass the summary produced by the same execution that emitted or suppressed rows.

Plan revision note, 2026-08-14: created the initial self-contained plan after resolving JSONL end-record and context-composition semantics with the user and related v3 issues.

Plan revision note, 2026-08-14 14:45Z: recorded completion of the implementation and focused-test milestone; clarified that ordinary human row output remains unchanged and the four human summary fields are rendered when `--summary` is requested.

Plan revision note, 2026-08-14 14:51Z: recorded completion of packaged documentation and skill guidance; docs-site generation and focused skill/runtime tests pass.