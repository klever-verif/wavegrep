# Make `change` a dense event table by default

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

After this change, `wavepeek change` produces one row for every event selected by the required `--on` expression unless the caller explicitly requests sparse rows. Callers can independently choose whether rows contain every requested signal or only values that changed. A user can observe the new behavior by running `wavepeek change` over repeated clock edges: the default output contains every selected edge, while `--row-mode sparse --row-values full` restores the version 2 behavior.

The two new options are `--row-mode dense|sparse`, defaulting to `dense`, and `--row-values full|delta`, defaulting to `full`. “Dense” means every selected event becomes a row. “Sparse” means only selected samples whose requested values differ from the previous selected sample become rows. “Full” means every requested signal appears in an emitted row. “Delta” means the first emitted row is full and later rows contain only requested signals whose current values differ from the previous selected sample; a dense delta row may therefore have an empty `signals` array.

## Non-Goals

This work does not add a separate `sample` command, change event-expression syntax, infer clocks, change `--sample-mode`, redesign output objects, or add schema-generation machinery removed from the repository. It does not alter `property` or extraction commands. It does not add dependencies or speculative abstractions.

## Progress

- [x] (2026-08-14 04:43Z) Read issue #85, repository guidance, current `change` execution paths, tests, and public documentation.
- [x] (2026-08-14 04:43Z) Resolve the pre-edge range-start ambiguity with the maintainer: skip an event when no pre-edge sample point exists.
- [x] (2026-08-14 05:04Z) Add the CLI mode types and carry them through every `change` engine.
- [x] (2026-08-14 05:04Z) Implement common row selection and signal projection semantics, including boundary, delta, truncation, and empty-result behavior.
- [x] (2026-08-14 05:04Z) Add and update unit and integration tests across output formats, engines, sampling modes, and waveform backends.
- [x] (2026-08-14 05:04Z) Update CLI help, bundled skill references, README, changelog, and migration guidance.
- [x] (2026-08-14 05:06Z) Run focused tests and `./dev just ci`, update this plan with evidence, and commit the implementation (commit remains the immediate next command).
- [x] (2026-08-14 05:42Z) Run Luna Max focused review wave and apply findings (completed: correctness, docs, architecture, and performance reviews; migration/help qualification, bounded edge-fast decode cache, centralized row emission, full FSDB mode parity, and full FST stream-mode parity; focused tests and 20 FSDB tests pass; review-fix commit is next).
- [x] (2026-08-14 05:50Z) Run Terra High focused review wave over the same areas, fix findings, test, and commit (completed: correctness, docs, architecture, and performance lanes; fixed delayed-value comparison, dense table wording, unnecessary dense/full baseline decoding, and sparse/full per-candidate flag allocation; focused tests and strict Clippy pass; commit is next).
- [ ] Run the independent Sol High control review, fix any findings, and run `./dev just check`.
- [ ] Remove this branch-local plan if required by repository policy, commit final cleanup, push, and open the pull request.

## Surprises & Discoveries

- Observation: `ChangeSnapshot.signals` is already a vector, and all human, JSON, and JSONL renderers already tolerate partial and empty vectors.
  Evidence: `src/contract/output.rs`, `src/contract/stream.rs`, and `src/output.rs` serialize or iterate the vector without a minimum-length requirement.

- Observation: `change` has baseline, fused, edge-fast, and pre-edge execution paths, so row semantics must be shared rather than patched into only the default path.
  Evidence: `src/engine/change.rs` dispatches from `run_with_sink` to `run_baseline_emit`, `run_fused_emit`, `run_edge_fast_emit`, or `run_pre_edge_emit`; `tests/change_opt_equivalence.rs` checks optimized-path equivalence.

- Observation: the repository no longer publishes JSON Schema artifacts.
  Evidence: issue #89 is recorded under `CHANGELOG.md` `[Unreleased]`; output vectors need no schema edit for delta rows.

- Observation: changing the default exposed a stale FSDB test expectation: VCD and FSDB both emitted the newly required dense row at 35ns, while the version 2 expected array ended at 15ns.
  Evidence: the first `./dev just ci` run passed 19 of 20 FSDB integration tests and failed only the literal expected array in `fsdb_change_json_matches_vcd_contracts`; the preceding VCD/FSDB parity assertion passed. Updating the expected dense row made `./dev just test-fsdb` pass all 20 tests.

- Observation: Luna performance review found that `IndexDecodeCache` became effectively unbounded under dense edge-fast scans because every selected event decoded requested values.
  Evidence: the cache key contains signal ID and candidate index; clearing it at the start of each candidate preserves within-candidate reuse while bounding retained entries. Focused engine-equivalence and backend tests pass after the change.

- Observation: Luna correctness review treated a representable pre-edge sample point with a requested signal lacking prior data as the same case as no representable pre-edge point.
  Evidence: `pre_edge_sample_time` already skips the maintainer-decided no-point case before sampling. `build_snapshot` has historically returned a signal error when an emit-eligible mixed sample lacks a value; changing all emitters to silently skip incomplete rows would broaden behavior beyond issue #85, so that suggestion was not applied.

- Observation: Terra correctness review found that the inherited comparison helper did not count a requested signal's first available `Some` value after a `None` baseline as a change.
  Evidence: changing comparison to full `Option` equality makes sparse/delta emit the first available value; `change_sparse_delta_emits_first_available_value` passes in baseline, fused, and forced edge-fast engines.

- Observation: Terra performance review found two compatibility-path costs: dense/full sampled an unused baseline, and sparse/full allocated a per-signal Boolean vector only to reduce it to one Boolean.
  Evidence: baseline values are now sampled only for sparse mode, dense/delta initializes state without decoding a baseline, dense/full holds no comparison vector, and `changed_values_and_update` collects flags only for delta output. Strict Clippy and focused equivalence tests pass.

## Decision Log

- Decision: Keep the existing `ChangeSnapshot` output type and renderers; project the signal vector before constructing a snapshot.
  Rationale: The existing contract already represents full, partial, and empty signal arrays, so a new DTO or output layer would add complexity without enabling behavior.
  Date/Author: 2026-08-14 / coding agent

- Decision: Compare each selected sample with the previous selected sample, and update comparison state even when sparse mode suppresses a row.
  Rationale: Issue #85 explicitly defines this order. It gives all engines one deterministic contract and avoids comparing against unrelated dump timestamps.
  Date/Author: 2026-08-14 / coding agent

- Decision: In pre-edge mode, skip a selected event when no representable sample point exists before it, including at `--from`.
  Rationale: The maintainer selected this behavior explicitly. It preserves the established pre-edge rule instead of substituting native values.
  Date/Author: 2026-08-14 / maintainer and coding agent

- Decision: Put migration guidance in `skills/wavepeek/references/change.md` and `CHANGELOG.md`, not a new standalone migration file.
  Rationale: The repository has no separate migration-document pattern; keeping the note next to the command contract is the smallest discoverable solution.
  Date/Author: 2026-08-14 / coding agent

- Decision: Describe `--row-mode sparse --row-values full` as restoring the version 2 row shape, not exact version 2 behavior.
  Rationale: Version 3 intentionally compares sparse samples with the previous selected sample, while version 2 used the preceding dump timestamp. Luna docs review correctly identified that gated triggers can therefore differ.
  Date/Author: 2026-08-14 / Luna review and coding agent

- Decision: Centralize row filtering, truncation, projection, and sink emission in one private `emit_row` helper.
  Rationale: Four engine-specific copies carried the public row contract and could drift. One direct helper fixes the shared source without adding a trait, strategy, module, or dependency.
  Date/Author: 2026-08-14 / Luna architecture review and coding agent

- Decision: Treat transitions between missing and available requested values as changes in selected-sample state.
  Rationale: Issue #85 defines comparison against the previous selected sample. A first available value differs from an unavailable baseline and must be eligible as the first full sparse/delta row; later waveform sampling normally retains available values, so the inverse remains an explicit build error only if a backend cannot provide an emit-eligible value.
  Date/Author: 2026-08-14 / Terra correctness review and coding agent

## Outcomes & Retrospective

The implementation and public documentation are complete. The final `./dev just ci` rerun passed 664 library tests, all integration suites, VCD/FST and JSON/JSONL parity, CLI contracts, documentation publication, 93.23% average source coverage with a 92.72% minimum dimension, and all 20 FSDB integration tests. The required review waves, final handoff gate, and PR remain.

## Context and Orientation

`wavepeek` is a Rust command-line program for deterministic VCD, FST, and optional FSDB waveform inspection. `src/cli/change.rs` defines command-line arguments using Clap. `src/engine/change.rs` resolves signals and event expressions, chooses an execution engine, evaluates selected timestamps, samples requested signal values, and emits `ChangeSnapshot` rows through a sink. The baseline engine is the general implementation; fused and edge-fast engines optimize common native-sampling workloads; pre-edge sampling uses values immediately before an edge while keeping the event timestamp as row `time`.

A “selected event” is a waveform timestamp at which the required `--on` expression evaluates true, including an optional `iff` condition. A “selected sample” is the vector of requested `--signals` sampled for that selected event according to `--sample-mode native|pre-edge`. A “comparison baseline” is the previous selected sample used to identify which values changed. The range is inclusive, but sparse mode treats the sample at `--from` as initialization rather than a change row. Dense mode emits a matching event at `--from` when sampling is possible. `--to` remains inclusive.

`src/contract/output.rs` defines `ChangeSnapshot`; `src/output.rs` renders human output, while the contract and stream modules serialize JSON and JSONL. These representations already permit any vector length. `tests/change_cli.rs` owns the main command behavior. `tests/change_opt_equivalence.rs` compares engines, `tests/change_vcd_fst_parity.rs` compares common waveform backends, `tests/fsdb_cli.rs` covers optional FSDB support, `tests/json_jsonl_parity.rs` covers machine-output parity, and `tests/cli_contract.rs` covers help and argument parsing. Unit tests in `src/engine/change.rs` and `src/tests/change_private_helpers.rs` construct `ChangeArgs` directly and must include the new fields.

Canonical user documentation lives under `skills/wavepeek/`. The main command guide is `skills/wavepeek/references/change.md`; related contracts are `command-model.md`, `machine-output.md`, `empty-results.md`, `clock-edge-sampling.md`, and `overview.md`. `skills/wavepeek/SKILL.md` is the package entry point. `README.md` and `CHANGELOG.md` provide repository-level public guidance.

All Cargo, test, formatting, lint, and quality commands run from the repository root through `./dev`. The standard behavior-change gate is `./dev just ci`; the final local handoff gate is `./dev just check`. Git and GitHub commands run on the host. Host hooks are installed in this worktree with `./dev --install-hooks`, and the worktree container must be running before committing.

## Open Questions

There are no blocking open questions. Exact empty-result wording will follow the acceptance language: dense mode reports that no selected events were found in the selected time range; sparse mode retains the existing statement that no signal changes were found in the selected time range.

## Plan of Work

First, add public Clap value enums `RowMode` and `RowValues` in `src/cli/change.rs`, each using kebab-case values and the required defaults. Add both fields to `ChangeArgs` under output options. Update detailed command help in `src/cli/mod.rs` and parsing assertions so the defaults and accepted values are observable.

Second, change `src/engine/change.rs` at the shared run dispatcher and each emitter. Introduce the smallest local helper that receives the current sampled values, mutable previous-selected values, whether any row has already been emitted, and the two mode values. It must determine changed indices, update the previous-selected state for every selected event, reject unchanged rows only in sparse mode, and return whether the snapshot should contain all indices or only changed indices. The first emitted delta row must use all indices. Avoid a trait or strategy abstraction because there are only two fixed enum dimensions.

The event loop order must be: evaluate `--on`; obtain a native or pre-edge sample; compare it with the previous selected sample; update comparison state; apply row-mode filtering; check `--max`; project full or delta signal values; emit. Dense mode must consider an event exactly at `--from`; sparse mode must use the range-start sample as a baseline and not emit it. If pre-edge sampling has no representable point, skip that event. The limit counts emitted rows after filtering, and truncation is reported only when an additional emit-eligible row exists.

Keep optimized native paths. Adapt fused and edge-fast candidate shortcuts so they do not discard selected events merely because requested signal offsets did not change when row mode is dense. Event expressions and `iff` still use their current native frame. Ensure optimized paths advance the same previous-selected comparison state as baseline, or fall back to baseline only where preserving an optimization would require duplicated semantic logic. Verify all forced engines produce identical rows for all four combinations.

Third, add behavior tests. In `tests/change_cli.rs`, use a small existing source-backed waveform with repeated selected edges and payloads that are unchanged, partially changed, and fully changed. Assert dense/full, dense/delta, sparse/full, and sparse/delta in human and JSON output. Assert the first emitted delta row is full, a later dense delta row can be empty, the `--from` dense/sparse distinction, inclusive `--to`, `iff`, `--max` after filtering, native and pre-edge sampling, and mode-specific empty diagnostics. Update old tests that intend version 2 behavior to pass `--row-mode sparse` explicitly.

Extend `tests/change_opt_equivalence.rs` across the four combinations and both candidate modes as appropriate. Extend `tests/change_vcd_fst_parity.rs`, `tests/fsdb_cli.rs`, and `tests/json_jsonl_parity.rs` with focused mode cases rather than duplicating the whole matrix. Update direct `ChangeArgs` literals and add a renderer unit assertion for partial and empty signal vectors if current tests do not already prove them. Keep fixtures minimal and reuse existing waveform sources where their event patterns suffice.

Fourth, update public text. Rewrite `skills/wavepeek/references/change.md` around dense default behavior, explain independent row selection and value projection, include the four combinations, and add migration guidance showing `--row-mode sparse --row-values full`. Update empty-result, machine-output, command-model, overview, and clock-edge references only where their statements would otherwise be false. Keep `SKILL.md` and `README.md` concise. Add a breaking `[Unreleased]` changelog item linked to issue #85. Do not create a schema or separate migration subsystem.

Finally, run formatting and focused tests, then the full CI gate. Commit the implementation with a conventional breaking-feature message. Run two focused review waves over code correctness, public contracts/docs, architecture/KISS, and performance. The first wave uses Luna Max reviewers; the second uses Terra High reviewers over the same lanes. Every reviewer is read-only and must apply KISS, YAGNI, and ponytail-review principles in addition to its lane. Apply substantive fixes centrally, rerun relevant tests, and commit each review wave. Run a fresh Sol High control review over the consolidated branch, apply any final fixes, run `./dev just check`, remove this branch-local plan unless maintainers need it retained, push, and open a PR against `dev3` with issue linkage and test evidence.

### Concrete Steps

Run all development commands from `/home/esynr3z/projects/wavepeek/.worktrees/wavepeek/85`.

Start the worktree container and ensure hooks are installed:

    ./dev --install-hooks
    ./dev true

During implementation, format and run focused tests:

    ./dev cargo fmt --all
    ./dev cargo test --test change_cli
    ./dev cargo test --test change_opt_equivalence
    ./dev cargo test --test change_vcd_fst_parity
    ./dev cargo test --test json_jsonl_parity
    ./dev cargo test --test cli_contract

Run optional FSDB tests through the repository recipe when the SDK is available:

    ./dev just test-fsdb

Before the implementation commit, run:

    ./dev just ci

Expect all formatting, lint, auxiliary, Rust coverage, build, skill, docs, and available FSDB checks to pass. Record concise totals or the final success lines in this plan.

Create conventional commits without bypassing hooks. The implementation commit should identify the breaking default in its footer:

    git commit -m "feat(change): add dense row modes" -m "BREAKING CHANGE: change now emits every selected event by default. Use --row-mode sparse --row-values full for version 2 behavior." -m "Closes #85"

After reviews and fixes, run the final local gate:

    ./dev just check

Push the host branch and open a PR against `dev3`:

    git push -u origin dev3-85/dense-change-table
    gh pr create --base dev3 --head dev3-85/dense-change-table --title "feat(change): add dense row modes" --body-file <prepared-pr-body>

### Validation and Acceptance

A detailed help invocation must list both options, accepted values, and defaults:

    ./dev cargo run -- change --help

A default `change` query over repeated matching edges must emit every sampled event with every requested signal. Adding `--row-values delta` must preserve every event while making the first row full and allowing later partial or empty `signals` arrays. Adding `--row-mode sparse` must suppress selected samples equal to the previous selected sample. Adding both sparse and delta must emit only changed samples, with the first emitted row full and later rows partial.

For a matching event at `--from`, dense mode must emit when native or pre-edge sampling is possible; sparse mode must not emit the range-start baseline. Both modes must include eligible events at `--to`. A pre-edge event with no representable prior sample point must be skipped. `--max 1` must count only rows that survive row-mode filtering. Empty dense and sparse queries must return success with their distinct `WPK-W0003` messages.

Human, JSON, and JSONL outputs must agree on rows and values. VCD and FST must match. FSDB must match the same contract when the optional feature and SDK are available. Forced baseline, fused, and edge-fast engines must agree for supported inputs. Existing meanings of `--on`, `iff`, `time`, `sample_time`, and `--sample-mode` must remain unchanged.

### Idempotence and Recovery

Formatting and test commands are safe to rerun. Source-backed generated VCD/FST and FSDB fixtures are ignored outputs and may be regenerated through existing `just` recipes. Do not delete arbitrary files in repository-root `tmp/`; use uniquely named files for PR bodies and logs. If an optimized engine diverges, first reproduce with its forced test mode and compare against baseline; use the existing baseline fallback only when the optimized preconditions genuinely do not support the request. Do not bypass hooks. If a commit hook fails, fix the reported issue and retry the normal commit.

Review workers are read-only. If one fails or returns no usable result, restart that lane with the same model tier and focus instead of treating it as passed. Apply all fixes in the main worktree, then rerun the directly affected tests before continuing.

### Artifacts and Notes

Issue #85 requires this behavior table:

    dense + full  => every selected event, all requested signals
    dense + delta => every selected event, first row full, later changed signals only
    sparse + full => changed selected samples only, all requested signals
    sparse + delta => changed selected samples only, first emitted row full, later changed signals only

The existing sparse empty diagnostic is:

    warning[WPK-W0003]: no signal changes found in selected time range

The planned dense diagnostic is:

    warning[WPK-W0003]: no selected events found in selected time range

### Interfaces and Dependencies

No dependency changes are required. In `src/cli/change.rs`, define two public copyable Clap enums:

    pub enum RowMode { Dense, Sparse }
    pub enum RowValues { Full, Delta }

Both derive `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `ValueEnum`, and `Default`; `Dense` and `Full` are defaults. `ChangeArgs` gains `pub row_mode: RowMode` and `pub row_values: RowValues`.

The implementation may add one private helper and one small private return representation inside `src/engine/change.rs` to centralize row selection and signal-index projection. It must not add a dependency, public trait, factory, strategy object, or new module. Existing `ChangeSnapshot`, sink traits, output envelopes, and renderers remain the public output interfaces.

Revision note (2026-08-14): Initial self-contained plan created after repository exploration and maintainer resolution of the no-pre-edge-sample boundary case.

Revision note (2026-08-14 05:04Z): Recorded completed implementation, tests, docs, focused validation, and the stale FSDB expected-output discovery before the final CI rerun.

Revision note (2026-08-14 05:06Z): Recorded the successful full CI rerun and exact validation evidence.

Revision note (2026-08-14 05:42Z): Recorded all Luna Max review findings, accepted fixes, one rejected out-of-scope behavior change, and focused post-fix validation.

Revision note (2026-08-14 05:50Z): Recorded Terra High review findings, architecture clean result, accepted fixes, and post-fix Clippy/equivalence evidence.
