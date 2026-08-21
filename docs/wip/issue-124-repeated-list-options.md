# Accept repeated list-valued CLI options

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

Wavepeek users can already pass comma-separated times, signals, and payload names. After this change they can instead repeat each list-valued option, or mix repeated and comma-separated occurrences, without changing output order, duplicate preservation, validation, or human/JSON/JSONL results. For example, `wavepeek value ... --at 10ns --at 20ns,30ns --signals a --signals b,c` must behave exactly like one flattened list in command-line order.

## Non-Goals

This work does not introduce a generic argument-merging layer, change scalar-option repetition, change protocol extractor `--map` or `--include` semantics, deduplicate values, or alter machine-output schemas.

## Progress

- [x] (2026-08-21 10:27Z) Read issue #124, repository guidance, CLI definitions, runtime normalization, tests, and public references.
- [x] (2026-08-21 10:27Z) Confirmed current behavior: repeated `--signals` and `--payload` already work through Clap-derived `Vec<String>` fields; repeated `--at` is rejected because it is a scalar `String`.
- [x] (2026-08-21 10:35Z) Changed `value --at` to the native Clap vector/delimiter pattern and updated runtime parsing.
- [x] (2026-08-21 10:35Z) Added focused behavior and help-contract tests for repeated, mixed, ordered, duplicate, and invalid input.
- [x] (2026-08-21 10:35Z) Updated packaged references and the Unreleased changelog.
- [x] (2026-08-21 10:40Z) Ran focused tests, `just ci`, and `just check`; all passed. Commit remains to be created after this plan update.
- [x] (2026-08-21) Ran parallel Luna Max review lanes, rebased the task onto current `origin/dev3`, rechecked all lanes, and resolved documentation findings. The requested plan cleanup remains intentionally scheduled after all review waves.
- [ ] Run parallel Terra High review lanes over the same areas, resolve findings, and commit any fixes.
- [ ] Run an independent Sol High control review, resolve findings, and commit any fixes.
- [ ] Remove this branch-local plan, run final gates, push the branch, and open a PR closing issue #124.

## Surprises & Discoveries

- Observation: Three requested option families need no parser implementation change because Clap infers append behavior for their existing `Vec<String>` fields.
  Evidence: current commands accepted repeated `value --signals`, `change --signals`, and `extract generic --payload`, including mixed comma-separated and repeated occurrences in declaration order.

- Observation: Existing empty-entry validation remains below Clap and reports command-specific errors.
  Evidence: `--signals top.anchor,,top.late` and `--payload anchor,,late` reach runtime normalization and fail with the existing “must not be empty” messages.

- Observation: Clap's default vector action appends repeated `--at` occurrences while `value_delimiter = ','` splits each occurrence, so no explicit `ArgAction` or merger is needed.
  Evidence: the manual mixed-form command emitted times `5ns`, `10ns`, `5ns` and signals `anchor`, `late`, `anchor` in exactly that order.

- Observation: The first `just ci` attempt found a direct parser unit assertion that focused integration tests did not compile under the library test configuration.
  Evidence: `src/cli/mod.rs` compared `args.at` to `"10ns"`; changing the expectation to `vec!["10ns"]` made the full gate pass. This was test-only fallout from the intentional field type change.

- Observation: The initial worktree history had diverged from current `origin/dev3`, so an `origin/main...HEAD` review showed 251 unrelated files.
  Evidence: the Luna Max minimal-design reviewer flagged `web/playground/index.md`; `git merge-base` showed the issue branch and current `origin/dev3` shared only old commit `72b5600`, while this task itself comprised two commits above `83317d2`.

- Observation: Current `origin/dev3` consolidated the packaged documentation since the original worktree base.
  Evidence: the old `command-model.md`, `value.md`, `change.md`, and `extract.md` files were deleted and their durable content now belongs in `commands.md`, `inspect-values.md`, `extract-transfers.md`, plus generated `cli-reference.md`.

- Observation: The corrected Luna Max wave found no code/test issues and identified only documentation precision, duplication, and planned WIP cleanup.
  Evidence: changelog wording now distinguishes newly repeatable `--at` from already-repeatable vector fields; shared flattening semantics remain only in `commands.md`; usage pages keep examples without duplicating the contract. Both docs and minimal-design lanes also confirmed this plan must be removed before PR.

## Decision Log

- Decision: Reuse the existing `value_delimiter = ','` and `Vec<String>` Clap pattern for `value --at` rather than add a merger helper.
  Rationale: Clap already flattens repeated and comma-separated occurrences in command-line order for the other affected fields.
  Date/Author: 2026-08-21 / coding agent

- Decision: Keep validation and trimming in the existing runtime normalization functions.
  Rationale: This preserves command-specific errors and keeps the CLI definition declarative.
  Date/Author: 2026-08-21 / coding agent

- Decision: Review in code/tests, user-facing docs/help, and minimal-design lanes; omit a performance lane.
  Rationale: The change affects parsing and public documentation but does not touch a meaningful hot path. Every lane must also apply KISS, YAGNI, and ponytail-review criteria.
  Date/Author: 2026-08-21 / coding agent

- Decision: Rebase only this task's two commits onto current `origin/dev3` and target the PR at `dev3`.
  Rationale: Issue #124 is assigned to the Wavepeek v3 milestone, the worktree and branch are `dev3`-scoped, and comparing against `main` includes the entire v3 development line rather than this issue.
  Date/Author: 2026-08-21 / coding agent

## Outcomes & Retrospective

Work is in progress. The expected final outcome is one native Clap list conversion, focused contract coverage for all four option families, aligned help/reference text, clean quality gates, two focused review waves, one independent control review, and an opened pull request.

## Context and Orientation

Wavepeek is a Rust command-line tool. Clap derives its argument parser from structs under `src/cli/`. `src/cli/value.rs` defines `ValueArgs`; its `at` field is currently a scalar `String`, while its `signals` field is a comma-delimited `Vec<String>`. `src/cli/change.rs` and `src/cli/extract.rs` use the same vector pattern for `--signals` and generic `--payload`. A list-valued option is an option whose values form one ordered sequence; “flattening” means concatenating values from comma-separated and repeated occurrences in the order the user wrote them.

`src/engine/value.rs` parses and validates requested time tokens before sampling. Signal and payload validation occurs in `src/engine/value.rs`, `src/engine/change.rs`, and `src/engine/extract.rs`. Integration behavior lives in `tests/value_cli.rs`, `tests/change_cli.rs`, and `tests/extract_generic_cli.rs`; help wording is asserted in `tests/cli_contract.rs`. Public, packaged documentation lives under `skills/wavepeek/references/`: `commands.md` owns shared command semantics, `inspect-values.md` owns value/change usage, `extract-transfers.md` owns generic extraction usage, and generated `cli-reference.md` mirrors live help. User-visible changes belong in `CHANGELOG.md` under `Unreleased`.

Repository commands run from the repository root through `./dev`. The standard full test gate is `./dev just ci`; the pre-handoff gate is `./dev just check`. Git hooks must be installed with `./dev --install-hooks` and the worktree container must be running before host commits.

## Open Questions

There are no open product questions. Issue #124 explicitly requires repetition, mixed forms, order and duplicate preservation, unchanged validation/output, help/tests for both forms, and no generalized parser framework.

## Plan of Work

First, change `ValueArgs::at` in `src/cli/value.rs` from `String` to required `Vec<String>` with comma delimiter and one-or-more values per occurrence, matching the existing signal fields. Change the time normalization function in `src/engine/value.rs` to consume the already flattened vector while retaining trimming, empty-token rejection, unit parsing, order, and duplicates. Update direct `ValueArgs` constructions in unit tests.

Second, add focused integration tests. `tests/value_cli.rs` must prove repeated and mixed `--at` and `--signals` forms produce ordered duplicate rows/columns and that malformed entries still fail. `tests/change_cli.rs` and `tests/extract_generic_cli.rs` must prove their already-supported repeated and mixed forms remain contracts. `tests/cli_contract.rs` must assert that help advertises comma-separated and repeated syntax for all four affected options.

Third, update long help in `src/cli/mod.rs`, field help in the CLI structs, packaged references under `skills/wavepeek/references/`, and `CHANGELOG.md`. Wording must describe one flattened ordered list and must not imply that a single comma-separated occurrence is required.

Fourth, run formatting, focused tests, `./dev just ci`, and `./dev just check`, then create a conventional commit. Run three parallel read-only Luna Max reviewers over code/tests, docs/help, and minimal design. Fix substantive findings and commit. Repeat those same lanes with fresh Terra High reviewers. Finally run one fresh read-only Sol High control reviewer over the complete diff. Fix any substantive finding, rerun affected gates, and commit.

At completion, update this plan with evidence and outcomes, remove it because `docs/wip/` artifacts are branch-local, rerun final gates if removal affects checks, push the branch to `origin`, and open a GitHub pull request against `dev3` with `Closes #124`.

### Concrete Steps

From the repository root, edit the files named above. During iteration run:

    ./dev cargo fmt --all -- --check
    ./dev cargo test --test value_cli --test change_cli --test extract_generic_cli --test cli_contract

Expect every test process to exit successfully. Exercise the user-visible path with the hand-written VCD fixture:

    ./dev cargo run --quiet -- value \
      --waves tests/fixtures/hand/value_delayed.vcd \
      --at 5ns --at 10ns,5ns \
      --signals top.anchor --signals top.late,top.anchor

Expect three rows in requested time order and three values per row in requested signal order, including duplicates. Then run:

    ./dev just ci
    ./dev just check

Both gates must exit successfully. Before each review wave, provide reviewers the issue, this plan, the `origin/dev3...HEAD` range, changed files, and gate results. Review agents are read-only and return severity, `file:line`, impact, and suggested fix, or “No substantive findings.”

After clean review and plan removal, run:

    git push -u origin dev3-124/repeated-list-options
    gh pr create --repo kleverhq/wavepeek --base dev3 \
      --head dev3-124/repeated-list-options \
      --title "feat(cli): accept repeated list options" \
      --body-file tmp/issue-124-pr.md

Expect GitHub CLI to print the new pull-request URL.

### Validation and Acceptance

Acceptance is observable when repeated and comma-separated occurrences of `value --at`, `value --signals`, `change --signals`, and `extract generic --payload` flatten in command-line order; duplicates remain visible; invalid empty or malformed entries still fail; a single occurrence behaves as before; and human, JSON, and JSONL data shapes do not change. Help and packaged references must explicitly mention both comma-separated and repeated forms. `./dev just ci` and `./dev just check` must pass, all required reviewers must return findings or a clean result, and the PR must be open against `dev3` with issue #124 linked for closure.

### Idempotence and Recovery

Edits and test commands are safe to repeat. The fixture command reads a checked-in VCD and creates no tracked output. If a gate fails, preserve its output under repository-root `tmp/`, fix only the reported cause, and rerun the narrow failing command before the full gate. Do not clean arbitrary files under `tmp/`. If a commit hook fails, fix the cause and retry without bypassing hooks. If push or PR creation fails, inspect remotes/authentication and retry the same host command; do not change repository code to compensate.

### Artifacts and Notes

Baseline evidence before implementation:

    wavepeek value ... --at 5ns --at 10ns
    fatal: args: the argument '--at <AT>' cannot be used multiple times

    wavepeek value ... --at 5ns --signals top.anchor --signals top.late
    @5ns top.anchor=1'h0 top.late=1'h1

This proves that only `--at` lacks native append behavior while the other vector fields already flatten repeated occurrences.

### Interfaces and Dependencies

No dependency changes are allowed or needed. Continue using Clap 4, already declared in `Cargo.toml`. At completion, `crate::cli::value::ValueArgs::at` must be `Vec<String>`, required by Clap and split with `value_delimiter = ','`. The value engine’s time normalization must accept the flattened sequence by reference and return the same parsed time representation it returns today. No new trait, helper module, configuration setting, or public output type should exist.

Revision note (2026-08-21): Completed the Luna Max wave after rebase, recorded its clean code result and documentation findings, and applied the changelog/duplication fixes. Terra High review remains.
