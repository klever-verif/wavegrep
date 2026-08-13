# Normalize JSON and JSONL result contracts

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

After this change, a consumer can process every successful WavePeek machine result without command-specific outer-shape branches. JSON always contains a `data` array, including the single metadata row from `info`; JSONL emits each corresponding array element as a `data` record. Protocol metadata is carried separately in top-level JSON `context` or JSONL `begin.context`.

A user can verify the result with `wavepeek info --waves <dump> --json`: `data` is a one-element array. The equivalent `--jsonl` stream starts with `begin`, emits one `data` record, and ends with record counts. The same correspondence holds for every waveform command.

## Non-Goals

This work does not change command-specific row fields, timestamp strings, ordered payload values, human output, expression behavior, protocol extraction behavior, JSON Schema support, summary behavior from issue #92, or fatal serialization from issue #104.

## Progress

- [x] (2026-08-12) Read issue #106, repository guidance, architecture, output contracts, current serializers, tests, and documentation.
- [x] (2026-08-12) Correct issue #106 so `info.data` is also an array.
- [x] (2026-08-13) Implement normalized JSON DTOs and focused unit tests.
- [x] (2026-08-13) Implement normalized JSONL records, writer counts, and streaming call-site updates.
- [x] (2026-08-13) Update integration tests for every command and add JSON/JSONL payload parity coverage.
- [x] (2026-08-13) Update public machine-output documentation and command examples.
- [x] (2026-08-13) Run focused tests and `./dev just ci`; commit the implementation.
- [x] (2026-08-13) Run Luna Max focused review wave, fix findings, and commit.
- [x] (2026-08-13) Run Terra High focused review wave over the same areas, fix findings, and commit.
- [ ] Run Sol High control review, resolve findings, run final gates, and commit cleanup.
- [ ] Push the branch and open a pull request against `dev3`.

## Surprises & Discoveries

- Observation: JSONL protocol context is already separated into `begin.context`, and JSONL row DTOs already reuse JSON row DTOs.
  Evidence: `src/contract/stream.rs` imports row types from `src/contract/output.rs`; protocol engines expose `context()` methods.
- Observation: issues #92 and #104 are open and absent from the current `dev3` base.
  Evidence: the former `EndRecord` had only its legacy summary, and fatal errors are still plain stderr output.
- Observation: the full CI gate includes native FSDB tests whose `info` assertions also depended on object-valued `data`.
  Evidence: the first `just ci` run passed coverage and regular integration tests, then failed two assertions in `tests/fsdb_cli.rs`; after indexing `data[0]`, `just test-fsdb` passed all 20 FSDB CLI tests.
- Observation: Luna review found that negative `time_precision` assertions silently weakened when `info.data` became an array, and the collected JSONL adapter lacked an explicit writer/result command check.
  Evidence: `.get("time_precision")` was called on the array rather than `data[0]`; an empty result could reach `begin` without row-level mismatch validation.

## Decision Log

- Decision: Make `info.data` a one-element array rather than retaining the issue's former object exception.
  Rationale: The maintainer clarified that the purpose is one shape across output modes and commands; every JSONL `data.data` must equal one JSON `data[]` element.
  Date/Author: 2026-08-12, maintainer and implementation agent.
- Decision: Reuse the existing engine result models and protocol context types; normalize only contract DTO conversion and stream transport.
  Rationale: Human rendering depends on current engine structures, while contract normalization does not require changing command execution.
  Date/Author: 2026-08-12, implementation agent.
- Decision: Do not pre-implement optional `summary` or fatal records.
  Rationale: Those belong to open issues #92 and #104, and issue #106 explicitly treats them as coordinated independent work.
  Date/Author: 2026-08-12, implementation agent.
- Decision: Do not add a separate FSDB-only JSONL smoke test suggested as a low-severity Terra finding.
  Rationale: JSONL dispatch and serialization are backend-independent after `info` returns its engine result; the existing all-command JSON/JSONL parity test exercises that exact path, while FSDB tests already verify the backend-specific `info` row. A duplicate test would not cover a distinct execution branch.
  Date/Author: 2026-08-13, implementation agent.

## Outcomes & Retrospective

The normalized contracts, command coverage, parity test, and packaged documentation are implemented. Review and PR stages remain.

## Context and Orientation

WavePeek is a Rust CLI. `src/cli/mod.rs` dispatches commands into `src/engine/`. Commands return `src/engine/mod.rs::CommandResult`, whose `CommandData` variants retain the native command result structures used by human output.

`src/contract/output.rs` converts a collected `CommandResult` into the strict JSON data transfer objects. Its current `OutputEnvelope` lacks `type`, and its protocol variants combine context fields with `events` or `transfers`. `info` currently serializes one object directly.

`src/contract/stream.rs` defines JSONL records. A stream currently emits `begin`, repeated `item` records, optional `diagnostic` records, and `end`. `item`, `diagnostic`, and `end` repeat `command`; the old end record stores `status`, item counts, and truncation under `summary`.

`src/output.rs::JsonlWriter` owns JSONL sequencing, flushing, row and diagnostic counts. `write_jsonl_result` adapts commands that first collect their rows. Streaming implementations in `src/engine/change.rs`, `property.rs`, `extract.rs`, `ahb.rs`, `apb.rs`, `atb.rs`, `axi.rs`, and `axistream.rs` call the same writer while executing.

Integration tests live under `tests/`. `tests/jsonl_cli.rs` centrally checks stream invariants. Protocol command suites additionally inspect their context and row payloads. Public machine-output guidance lives in `skills/wavepeek/references/machine-output.md`; command examples are in sibling reference files.

In this plan, a row means one command-specific serialized result element. For `info`, the sole metadata object is one row. Context means protocol-wide metadata that applies to every extracted event or transfer.

## Open Questions

There are no blocking questions. The corrected GitHub issue is authoritative where it differs from the original text: `data` is always an array.

## Plan of Work

First, change `src/contract/output.rs`. Add the constant result type to `OutputEnvelope`, add an optional protocol context, and convert every `OutputData` variant to an array. Keep all existing row DTOs unchanged. Reuse the context DTO definitions used by JSONL rather than creating parallel protocol metadata structures. Split each protocol engine result into its existing `context()` value and row collection only while building the envelope.

Second, change `src/contract/stream.rs` and `src/output.rs`. Rename item transport concepts to data transport concepts. A data record contains only `type`, `seq`, and `data`; a diagnostic record contains only `type`, `seq`, and `diagnostic`; an end record contains `type`, `seq`, and `records` with data and diagnostic counts. Keep `command` and optional context only in `begin`. Remove the obsolete truncation calculation and update the eight streaming engines mechanically to call `data` and parameterless `end`.

Third, update tests. Unit tests must assert exact envelope and record shapes, including one-element `info.data`, protocol context separation, absent repeated commands, interleaved data and diagnostics, contiguous sequence numbers, and end counts. Integration tests must preserve command-specific row assertions while adopting `data.data`. Add a shared JSON versus JSONL parity path that exercises all twelve serialized commands, compares context, rows, and diagnostics, and handles `info` identically to all other one-row results. Retain empty-stream coverage.

Fourth, update `skills/wavepeek/references/machine-output.md` and obsolete examples in command references. State one invariant: JSON `data` is always an array and every JSONL `data.data` is one array element. Document protocol context placement and record counts without describing #92 or #104 as already shipped.

Finally, run the repository gates and three required review stages. Luna Max reviewers inspect correctness/tests, contracts/architecture, and docs in parallel. Terra High reviewers repeat those same areas independently after Luna findings are fixed. Sol High performs one fresh control review of the consolidated branch. Every reviewer is read-only and must apply KISS, YAGNI, and ponytail-review principles alongside its assigned focus. Fix substantive findings in the main session, rerun affected checks, remove this branch-local plan, push, and open a PR against `dev3`.

### Concrete Steps

Run all commands from `/home/esynr3z/projects/wavepeek/.worktrees/wavepeek/106`.

Install hooks once before commits:

    ./dev --install-hooks

During implementation, use focused checks:

    ./dev cargo fmt --all -- --check
    ./dev cargo test --lib contract::
    ./dev cargo test --test jsonl_cli

Search for obsolete shapes before documentation completion:

    rg 'item\.item|"type":"item"|data\.(events|transfers)|summary.*items|\["item"\]' src tests skills/wavepeek/references

Before review and handoff, run:

    ./dev just ci
    ./dev just check

Expected final evidence is that both commands succeed, all review waves report no unresolved substantive findings, and `git status --short` is clean after the final commit.

### Validation and Acceptance

Every successful `--json` waveform command must emit an object with `type: "result"`, its command name, optional protocol context, array-valued `data`, and array-valued diagnostics. `info.data` must contain exactly one metadata object. Empty row commands must contain `data: []`.

Every successful `--jsonl` stream must start at sequence zero with `begin`, carry command only there, optionally carry protocol context there, emit command rows as `data.data`, emit diagnostics as `diagnostic.diagnostic` in any position between begin and end, and terminate with exact emitted data and diagnostic record counts. Corresponding JSON array elements and JSONL data payloads must compare equal for all commands.

No output may gain `$schema`, numeric timestamps, reordered path/value structures, changed command-specific fields, or issue #92/#104 behavior. Human tests and the full CI gate must remain green.

### Idempotence and Recovery

All edits and validation commands are safe to repeat. The devcontainer command starts or reuses the worktree-specific container. If a focused test fails, preserve its output under repository `tmp/`, fix only the shared source of the mismatch, and rerun that test before broader gates. Git commits provide recovery points; do not bypass hooks.

The execution plan is branch-local and must be removed in the final cleanup commit while retaining `docs/wip/AGENTS.md`.

### Artifacts and Notes

The corrected issue is `https://github.com/kleverhq/wavepeek/issues/106`. Its key invariant is:

    JSON data is always an array.
    JSONL data.data is one corresponding element of that array.

The intended `info` JSON shape begins:

    {"type":"result","command":"info","data":[{"time_unit":"1ns",...}],"diagnostics":[]}

The intended end record is:

    {"type":"end","seq":2,"records":{"data":1,"diagnostics":0}}

### Interfaces and Dependencies

Use existing `serde` and `serde_json`; add no dependencies. Preserve `crate::engine::CommandResult`, `CommandData`, protocol data structures, and human rendering interfaces.

At completion, `crate::contract::output::OutputEnvelope` must serialize `type`, `command`, optional `context`, array-valued `data`, and diagnostics. `crate::contract::stream` must expose begin, data, diagnostic, and end record DTOs. `crate::output::JsonlWriter` must expose `begin`, `begin_context`, `data`, `diagnostic`, and parameterless `end`, while retaining one-line flushing and broken-pipe behavior.

Revision note (2026-08-12): Initial plan created after repository and issue inspection; it incorporates the maintainer correction that `info.data` is a one-element array and records the required staged review process.

Revision note (2026-08-13): Updated progress and discoveries after implementation, parity coverage, documentation updates, and the first full CI attempt exposed FSDB-specific `info.data` assertions.

Revision note (2026-08-13): Recorded Luna Max review findings and fixes: restored negative info-row assertions, validated collected JSONL command identity, and corrected stale architecture and sampling documentation.

Revision note (2026-08-13): Recorded the clean Terra architecture/docs passes and the decision not to duplicate backend-independent JSONL coverage in the FSDB suite.
