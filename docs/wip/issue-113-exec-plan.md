# Normalize scope-relative human signal paths

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

After this change, people can mix scope-relative and canonical signal references in `value`, `change`, and `extract generic` without receiving mixed path styles in terminal output. With `--scope top.cpu`, both `valid` and `top.cpu.valid` render as `valid`, while `core.valid` and `top.cpu.core.valid` render as `core.valid`. Passing `--abs` still renders canonical paths, and JSON or JSONL still exposes canonical `path` fields.

The behavior is observable through the integration tests in `tests/value_cli.rs`, `tests/change_cli.rs`, and `tests/extract_generic_cli.rs`, followed by the repository quality gate `./dev just ci`.

## Non-Goals

This work does not add relative-path fields to JSON or JSONL, change signal resolution or scope semantics, alter protocol extractors, change ordering or duplicate handling, or introduce a second path-normalization implementation. GitHub issue #114 owns future machine-output scope context.

## Progress

- [x] (2026-08-14 14:16Z) Read issue #113, repository guidance, execution paths, public contracts, and existing tests.
- [x] (2026-08-14 14:16Z) Start the worktree devcontainer and install worktree-local Git hooks.
- [x] (2026-08-14 14:24Z) Add focused regression tests that demonstrate normalized default output and preserved `--abs` output for all three commands.
- [x] (2026-08-14 14:24Z) Reuse the existing canonical-to-relative helper in the three generic command engines.
- [x] (2026-08-14 14:24Z) Update the packaged command-model contract.
- [x] (2026-08-14 14:28Z) Run focused tests and `./dev just ci`, then commit the implementation (focused result: 52 change, 14 extract-generic, and 28 value tests passed; full gate passed with 93.20% average source coverage and all available FSDB gates).
- [ ] Run first-wave focused reviews with Luna Max, fix findings, and revalidate.
- [ ] Run second-wave focused reviews with Terra High over the same lanes, fix findings, and revalidate.
- [ ] Run an independent Sol High control review, fix findings, and revalidate.
- [ ] Record the outcome, remove this branch-local plan, commit cleanup, push, and open a pull request closing issue #113.

## Surprises & Discoveries

- Observation: The waveform layer already contains a boundary-safe helper named `display_signal_path`; no new normalization logic is needed.
  Evidence: `src/waveform/mod.rs` strips the exact selected scope and then a separating dot, otherwise returning the canonical path unchanged.
- Observation: Human and machine output already use separate fields.
  Evidence: `display` is marked `#[serde(skip_serializing)]` while `path` remains canonical in the value, change, and extract payload result types.
- Observation: Duplicate behavior differs intentionally by command and must not be unified.
  Evidence: `value` and `change` preserve requested entries, while `extract generic` calls `require_unique_payloads` on canonical paths and rejects duplicates.
- Observation: The new human-output assertion failed on the unmodified implementation exactly at mixed canonical input spelling.
  Evidence: Before the engine edits, `change_abs_only_affects_human_labels_not_json_payload` produced `top.clk` and `top.cpu.valid` in default output where the test expected `clk` and `cpu.valid`; after the edits all 94 focused integration tests passed.

## Decision Log

- Decision: Normalize `display` only after a signal has resolved to a canonical path.
  Rationale: This directly satisfies issue #113 while preserving the existing input-resolution and diagnostic query-name paths.
  Date/Author: 2026-08-14 / pi
- Decision: Make `crate::waveform::display_signal_path` visible only within the crate and reuse it from engine modules.
  Rationale: The helper already implements the required exact-prefix behavior; `pub(crate)` is the narrowest visibility that permits reuse without creating a public API.
  Date/Author: 2026-08-14 / pi
- Decision: Leave `src/output.rs` unchanged.
  Rationale: Human rendering already selects `display` by default and canonical `path` under `--abs`; the defect is incorrect engine data, not rendering.
  Date/Author: 2026-08-14 / pi
- Decision: Keep each command's current ordering and duplicate semantics.
  Rationale: Issue #113 explicitly preserves these contracts; normalization changes presentation only.
  Date/Author: 2026-08-14 / pi

## Outcomes & Retrospective

Implementation and validation are complete. Completion still requires two focused review waves, one independent control review, cleanup of this plan, and an opened pull request.

## Context and Orientation

`wavepeek` is a Rust command-line tool. The CLI layer parses arguments, an engine module resolves and samples waveform signals, and `src/output.rs` renders the resulting data. A canonical path is the full dump-derived name such as `top.cpu.core.valid`. A scope-relative path omits the selected scope prefix, producing `core.valid` under `--scope top.cpu`.

`src/engine/value.rs` builds `ValueSignalValue` records. `src/engine/change.rs` builds `ChangeSignalValue` records. `src/engine/extract.rs` resolves generic extractor payloads into `PayloadSignal` and later builds `ExtractPayloadValue` records. Each result has a human-only `display` field and a serialized canonical `path` field. `src/output.rs` already prints `display` normally and `path` with `--abs`.

`src/waveform/mod.rs::display_signal_path` already converts a canonical path to a scope-relative path using an exact scope prefix and dot boundary. It is currently private because only waveform signal-listing diagnostics use it. The implementation should expose it as `pub(crate)` and import it directly in the three affected engines.

The public naming contract is in `skills/wavepeek/references/command-model.md`. Integration tests execute the compiled CLI against small waveform fixtures. Repository tools must run inside the devcontainer through root `./dev`; Git operations and GitHub operations run on the host.

## Open Questions

There are no blocking product questions. The phrase “duplicates remain unchanged” is interpreted as preserving each command's existing behavior: `value` and `change` retain duplicate entries and ordering, while `extract generic` continues rejecting duplicate canonical payload paths.

## Plan of Work

First, extend existing mixed-name and human-output integration tests rather than creating fixtures or test files. Each affected command must exercise both direct children and nested descendants under a selected scope, mixing relative and in-scope canonical inputs. Assertions must prove that default output consistently omits the scope and that `--abs` consistently restores canonical paths. Existing JSON and JSONL tests remain the contract guard because production serialization is not changed.

Next, change `src/waveform/mod.rs::display_signal_path` from private to `pub(crate)`. In `src/engine/value.rs`, preserve user tokens for resolution diagnostics but build each output `display` from the sampled canonical path and selected scope. In `src/engine/change.rs`, normalize the stored requested display after canonical path construction so all emitted snapshots reuse it while diagnostic query names remain user-authored. In `src/engine/extract.rs`, normalize `PayloadSignal.display` from each resolved canonical path during payload binding. Do not change renderers, resolution helpers, protocol engines, result serialization, dependencies, or architecture documentation.

Update section 4 or 5 of `skills/wavepeek/references/command-model.md` with the precise human-output rule: `value`, `change`, and `extract generic` derive displayed paths from canonical paths, render them relative to `--scope`, and render canonical paths without a scope or under `--abs`. Keep machine-output wording unchanged.

Run formatting and the three focused integration test targets. Then run `./dev just ci`. Commit the implementation with a conventional `fix` commit that closes #113 in its footer.

Review the committed diff in three lanes: correctness/tests, public documentation/contracts, and architecture/performance/simplicity. Every reviewer must apply KISS, YAGNI, and ponytail-review principles in addition to its lane focus. Launch the first three reviewers in parallel using Luna Max, then launch three fresh reviewers over the same lanes in parallel using Terra High. Apply findings in the main session, rerun affected tests and the full gate, and commit fixes when needed. Finally launch a fresh Sol High reviewer for a consolidated control review across all lanes. A clean response must say there are no substantive findings; otherwise fix and revalidate before proceeding.

At completion, update this document with evidence and outcomes, then remove it because `docs/wip/` artifacts do not ship. Commit that cleanup, push the branch, and open a GitHub pull request against `dev3` with a concise summary, test evidence, review evidence, and `Closes #113`.

### Concrete Steps

Run all commands from `/home/esynr3z/projects/wavepeek/.worktrees/wavepeek/dev3-113`.

Inspect and edit:

    src/waveform/mod.rs
    src/engine/value.rs
    src/engine/change.rs
    src/engine/extract.rs
    tests/value_cli.rs
    tests/change_cli.rs
    tests/extract_generic_cli.rs
    skills/wavepeek/references/command-model.md

Validate the focused slice:

    ./dev cargo fmt --all -- --check
    ./dev cargo test -q --test value_cli --test change_cli --test extract_generic_cli

Expected result: all three integration-test binaries pass, including new mixed relative/canonical nested-path assertions.

Validate the repository:

    ./dev just ci

Expected result: all formatting, lint, coverage, build, packaged-skill, docs-site, auxiliary, and available FSDB gates pass or report an explicit supported skip.

Inspect and commit on the host:

    git diff --check
    git status --short
    git diff
    git add <changed files>
    git commit -m "fix(output): normalize scoped signal paths" -m "Closes #113"

After reviews and any fix commits, clean up and publish:

    git rm docs/wip/issue-113-exec-plan.md
    git commit -m "chore(wip): remove completed issue plan"
    git push -u origin dev3-113/normalize-scope-relative-paths
    gh pr create --repo kleverhq/wavepeek --base dev3 --head dev3-113/normalize-scope-relative-paths --title "fix(output): normalize scoped signal paths" --body-file <prepared PR body>

### Validation and Acceptance

The change is accepted when `value`, `change`, and `extract generic` all show `valid` for both `valid` and `top.cpu.valid` under `--scope top.cpu`, and show `core.valid` for both nested input forms. The same invocations with `--abs` must show `top.cpu.valid` and `top.cpu.core.valid`. Without `--scope`, human paths remain canonical. JSON and JSONL tests must continue showing canonical `path` fields. Ordering and duplicate tests must remain unchanged and pass.

The implementation must pass the three focused integration-test targets and `./dev just ci`. It must also receive completed reviews from all requested Luna Max, Terra High, and Sol High reviewers, with every substantive finding resolved or explicitly documented before the pull request opens.

### Idempotence and Recovery

The test and quality commands are safe to repeat. `./dev --install-hooks` is idempotent. If an edit fails, restore only this task's file with `git checkout -- <path>` after inspecting `git diff`; do not reset unrelated user work. If a hook fails, fix the cause and retry the commit without bypassing hooks. If push or PR creation fails, preserve local commits and retry the host command after checking authentication and remote state.

### Artifacts and Notes

Issue #113 requires this representative conversion:

    command: wavepeek value --scope top --signals cpu.valid,top.cpu.ready
    old:     cpu.valid=... top.cpu.ready=...
    new:     cpu.valid=... cpu.ready=...

The central helper has the effective contract:

    display_signal_path("top.cpu.core.valid", Some("top.cpu")) == "core.valid"
    display_signal_path("top.cpu.core.valid", None) == "top.cpu.core.valid"

### Interfaces and Dependencies

No dependency changes are allowed or needed. At completion, `src/waveform/mod.rs` exposes this existing helper within the crate:

    pub(crate) fn display_signal_path<'a>(
        canonical_path: &'a str,
        scope: Option<&str>,
    ) -> &'a str

The three engine modules call this helper when assigning human-only `display` values. Existing serialized result types and command interfaces remain unchanged.

Revision note (2026-08-14): Created the self-contained implementation and review plan after repository and issue investigation. The plan chooses reuse of the existing waveform helper as the smallest contract-preserving implementation.

Revision note (2026-08-14 14:24Z): Recorded the completed implementation slice, focused red/green test evidence, and the remaining full quality gate.

Revision note (2026-08-14 14:28Z): Recorded the passing full CI gate, including source coverage and FSDB validation.
