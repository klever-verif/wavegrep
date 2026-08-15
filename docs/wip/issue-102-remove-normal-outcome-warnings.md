# Remove diagnostics for intentional unlimited and empty results

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

WavePeek currently labels two successful, expected outcomes as warnings: a user explicitly requesting an unlimited result and a valid query returning zero rows. After this change, those outcomes remain successful but no longer produce `WPK-W0001` or `WPK-W0003`. JSON and JSONL clients use the existing result summary to identify unlimited and empty results, while human users receive a short normal-output message on stdout for empty results. Truncation and lossy or unexpected conditions keep their existing diagnostics.

A user can observe the change by running an empty `signal` query in human, JSON, and JSONL modes. Human mode prints a short `no signals found in selected scope` line with empty stderr. JSON returns `data: []`, a zero-result summary, and `diagnostics: []`. JSONL returns `begin` and `end` records with no diagnostic record and a zero-result summary. An invocation with `--max unlimited` reports `summary.limit: null` in machine output without a diagnostic.

## Non-Goals

This work does not add or restore the removed `docs search` command even though issue #102 mentions it. It does not change fatal handling for invalid arguments, scopes, signal references, or expressions. It does not alter result selection, limits, summaries, output schemas, stream structure, or diagnostic codes `WPK-W0002`, `WPK-W0004`, and `WPK-W0005`. It does not reserve retired codes through placeholder enum variants; their absence and the unchanged explicit strings for later codes prevent reassignment.

## Progress

- [x] (2026-08-15 04:58Z) Read issue #102, repository guidance, output contracts, affected engine paths, tests, and bundled skill references.
- [x] (2026-08-15 04:58Z) Confirmed with the maintainer that the absent `docs search` command is out of scope.
- [x] (2026-08-15 05:13Z) Removed `WPK-W0001` and `WPK-W0003` production paths while preserving summaries and all other diagnostics.
- [x] (2026-08-15 05:13Z) Added normal human stdout messages for empty discovery and row-producing results.
- [x] (2026-08-15 05:13Z) Updated integration and unit tests for human, JSON, and JSONL behavior across discovery, change, property, and extract commands.
- [x] (2026-08-15 05:13Z) Updated CLI help, changelog, public contracts, and bundled skill references.
- [x] (2026-08-15 05:37Z) Ran focused tests, auxiliary suites, and final `./dev just ci` after all review fixes; all passed, including FSDB and 90% coverage gates.
- [x] (2026-08-15 05:19Z) Ran parallel Luna Max reviews by code/test, docs/contract, and simplicity/architecture focus; fixed one stale property guide sentence and two contract wording ambiguities.
- [x] (2026-08-15 05:26Z) Ran parallel Terra High reviews over the same areas; qualified `--summary` documentation, added an empty human-summary regression test, and removed empty diagnostic-vector plumbing from generic extraction.
- [x] (2026-08-15 05:35Z) Ran an independent Sol High control review; added representative empty-unlimited JSONL coverage, replaced retired codes in benchmark fixtures, and received a clean Sol High re-review.
- [ ] Remove this completed branch-local plan, commit cleanup, push, and open a PR for issue #102 (implementation, reviews, and final gates are complete).

## Surprises & Discoveries

- Observation: issue #102 names `docs search`, but the command was removed by issue #77 before this branch point.
  Evidence: `src/cli/mod.rs` contains only the `skill` helper, and the Unreleased changelog records removal of embedded topic browsing and search.

- Observation: human rendering previously returned an empty string for empty row collections, so merely deleting `WPK-W0003` would make successful empty output indistinguishable from missing output.
  Evidence: `src/output.rs::render_human_with_data` joined empty iterators, while `output::write` writes stdout only when that rendered string is non-empty.

- Observation: all non-protocol empty human collections converge in `src/output.rs::render_human_with_data`; protocol renderers already emit context and an empty section.
  Evidence: a single fallback match adds five messages in 17 changed lines, avoiding per-engine output state or changes to protocol renderers.

- Observation: the first review wave found no code, test, simplicity, or architecture defects, but found one stale `WPK-W0001` sentence and wording that could overstate empty-result diagnostic suppression and normal-human output under `--summary`.
  Evidence: Luna Max code/test, docs/contract, and simplicity/architecture lanes completed; the stale and ambiguous statements were corrected in the property guide, machine-output contract, CLI help, and help tests.

- Observation: the second review wave found no correctness defect, but found that command references did not consistently qualify `--summary`, the suppression lacked a direct empty human regression test, and generic extraction still forwarded an always-empty diagnostics vector.
  Evidence: Terra High code/test, docs/contract, and simplicity/architecture lanes completed; documentation now separates ordinary and summary-only output, the test locks summary-only stdout, and the vector is initialized only where truncation can add a diagnostic.

- Observation: the control review found missing representative empty-unlimited JSONL coverage and retired synthetic codes in benchmark tests, then accepted the fixes without further findings.
  Evidence: JSONL now covers change, scope, property, and generic extraction paths with zero rows, no diagnostics, and `limit: null`; all 93 benchmark E2E unit tests pass; the second Sol High pass reported `No substantive findings.`

## Decision Log

- Decision: Ignore `docs search` rather than adding or restoring it.
  Rationale: The maintainer explicitly confirmed this interpretation, and adding an unrelated command would violate KISS and YAGNI.
  Date/Author: 2026-08-15 / pi

- Decision: Use concise command-specific human messages, retaining existing wording where the renderer has enough context and using `no change rows found in selected time range` for both change row modes.
  Rationale: The renderer does not carry row-mode configuration after execution. One accurate message avoids adding output state solely to preserve obsolete diagnostic wording.
  Date/Author: 2026-08-15 / pi

- Decision: Put empty-result text in the existing human renderer and keep `--summary` output summary-only.
  Rationale: Empty text is an output-format concern, not an engine diagnostic. The renderer already owns command-specific human shapes, and summary-only mode intentionally suppresses data rendering while showing numeric evidence of emptiness.
  Date/Author: 2026-08-15 / pi

- Decision: Delete the retired enum variants and keep later warning strings explicit and unchanged.
  Rationale: Placeholder variants are dead code. Existing explicit mappings retain `WPK-W0002`, `WPK-W0004`, and `WPK-W0005`, so removing two variants cannot renumber them.
  Date/Author: 2026-08-15 / pi

## Outcomes & Retrospective

Implementation and all required review waves are complete. `WPK-W0001`, `WPK-W0003`, `LimitDisabled`, and `EmptyResult` are absent from production, tests, benchmark fixtures, CLI help, changelog, and packaged skill references. Human empty discovery and row output now uses normal stdout text when rows are requested; `--summary` stays summary-only. JSON and JSONL retain zero-result summaries and empty data without an empty-result diagnostic, and unlimited `--max` is represented by `limit: null`.

Focused suites, 93 auxiliary Python tests, pre-commit hooks, and final `./dev just ci` passed. The final CI coverage report was 93.12% regions, 92.55% functions, and 93.70% lines; FSDB-enabled checks and 23 FSDB integration tests also passed. Luna Max and Terra High reviewed code/tests, docs/contracts, and simplicity/architecture in two waves. Sol High found two final test-fixture gaps, accepted their fixes on a fresh control pass, and reported no substantive findings.

No product limitation remains for issue #102. The removed `docs search` command was intentionally ignored as agreed with the maintainer. Only branch cleanup, push, and PR creation remain.

## Context and Orientation

The repository is a Rust CLI. `src/cli/mod.rs` parses commands and dispatches them through `src/engine/mod.rs`. The command engines in `src/engine/scope.rs`, `signal.rs`, `change.rs`, `property.rs`, `extract.rs`, and `ahb.rs` build result rows, a `ResultSummary`, and a vector of non-fatal diagnostics. APB, ATB, AXI, and AXI-Stream wrappers reuse the shared extraction execution paths. `src/diagnostic.rs` maps warning variants to stable public codes. `src/output.rs` renders human output and serializes JSON or JSONL output.

A result summary describes the selected public rows. `complete` states whether execution found the end of the selected set, `returned` counts accepted rows, `limit` is the numeric `--max` or JSON null for unlimited, and `total` is exact when known. This model already represents the two normal outcomes being changed, so no new metadata is needed.

The integration tests in `tests/scope_cli.rs`, `signal_cli.rs`, `change_cli.rs`, `change_opt_equivalence.rs`, `property_cli.rs`, `extract_generic_cli.rs`, `extract_apb_cli.rs`, `jsonl_cli.rs`, and `fsdb_cli.rs` contain direct expectations for the retiring diagnostics. `tests/result_summary_cli.rs` and `tests/json_jsonl_parity.rs` cover cross-format summaries and parity. Public behavior is documented in the embedded long help in `src/cli/mod.rs` and the packaged skill under `skills/wavepeek/references/`, especially `command-model.md`, `machine-output.md`, `empty-results.md`, and the command guides. `CHANGELOG.md` records the user-visible removal.

## Open Questions

There are no blocking product questions. Protocol-specific human rendering remains unchanged because its context and empty row-section header already distinguish a successful empty result.

## Plan of Work

First remove only the diagnostic construction that represents intentional unlimited limits or valid empty rows. In `src/diagnostic.rs`, remove `LimitDisabled` and `EmptyResult` and their explicit code mappings. In each affected engine, initialize or retain diagnostics only when another diagnostic can actually be produced, such as truncation, unmatched protocol candidates, or ambiguous FSDB paths. Do not touch summary calculation or fatal validation.

Next update `src/output.rs::render_human_with_data` so empty `Scope`, `Signal`, `Change`, `Property`, and extraction collections produce their existing descriptive text as stdout in normal human mode. Preserve change dense-versus-sparse wording, extraction protocol context, and `--summary` behavior. Prefer direct empty checks in existing match arms over a new abstraction because each command owns a distinct data shape or message.

Then update tests. Replace expectations for `WPK-W0001` and `WPK-W0003` with empty diagnostic arrays or no diagnostic records. Assert empty stderr and the normal human stdout message. Ensure representative human, JSON, and JSONL cases cover discovery, change, property, generic extraction, and protocol extraction, while existing truncation and fatal tests remain unchanged. Update FSDB expectations where the shared engine behavior changes, without adding format-specific behavior.

Finally update all public wording. Remove references that say unlimited or empty success emits a diagnostic from `src/cli/mod.rs` and the packaged skill references. Describe unlimited as represented by `summary.limit: null`, empty machine output as represented by rows and summary, and empty human output as a short stdout message. Add one concise Unreleased changelog entry under Removed. Search the entire tracked tree to prove the retired codes and stale descriptions are gone.

After implementation, run the required three-stage review. The first wave uses Luna Max in parallel code/test, docs/contract, and simplicity/architecture lanes. The second wave uses Terra High over the same lanes after Luna findings are resolved. A fresh Sol High reviewer then performs a consolidated control review. Every reviewer is read-only, reports concrete findings with file and line references, and applies KISS, YAGNI, and ponytail-review principles in addition to its lane focus. The main agent applies fixes and reruns relevant checks.

### Concrete Steps

Run all commands from the repository root `/home/esynr3z/projects/wavepeek/.worktrees/wavepeek/102`.

Install hooks and keep the worktree container running before commits:

    ./dev --install-hooks

Use repository searches before and after edits:

    rg -n 'LimitDisabled|EmptyResult|WPK-W0001|WPK-W0003' src tests skills CHANGELOG.md
    rg -n 'disabled-limit|empty-result|explicitly disabled|emits.*WPK-W000[13]' src skills

Run focused formatting and affected integration tests while iterating:

    ./dev just format
    ./dev cargo test --test scope_cli --test signal_cli --test change_cli --test change_opt_equivalence --test property_cli --test extract_generic_cli --test extract_apb_cli --test jsonl_cli --test result_summary_cli

Run the complete behavior gate before review and after final fixes:

    ./dev just ci

Expected focused and complete test commands exit zero. The final retired-code search returns no matches except the historical plan while it exists; after plan cleanup it returns no tracked matches. Truncation searches and tests continue to show `WPK-W0002`.

Commit logical milestones with conventional messages through installed hooks. Do not bypass hooks. After review and plan cleanup, push the branch and open a PR:

    git push -u origin dev3-102/remove-limit-empty-warnings
    gh pr create --repo kleverhq/wavepeek --base dev3 --head dev3-102/remove-limit-empty-warnings

The PR body must summarize behavior, tests, review waves, and close issue #102.

### Validation and Acceptance

A human empty discovery query exits zero, prints a concise command-specific empty message to stdout, and leaves stderr empty. The same valid query with `--json` has an empty data array, `summary.complete: true`, `summary.returned: 0`, the selected numeric limit, `summary.total: 0`, and `diagnostics: []`. With `--jsonl`, it emits no diagnostic record; the end record has `records.diagnostics: 0` and the same summary.

Representative change, property, generic extract, and protocol extract zero-row queries have the same format semantics. `--max unlimited` and applicable `--max-depth unlimited` invocations emit no diagnostic; machine summaries use `limit: null` for unlimited `--max`. Invalid query inputs retain non-zero fatal behavior. A genuinely truncated result still emits only `WPK-W0002`, and ambiguity or lossy extraction continues using `WPK-W0004` or `WPK-W0005` where applicable.

The tracked source, tests, CLI help, changelog, and packaged skill contain no `WPK-W0001`, `WPK-W0003`, `LimitDisabled`, or `EmptyResult` after branch-plan cleanup. `./dev just ci` passes. All required reviewers either report no substantive findings or their findings are fixed and rechecked before the PR is opened.

### Idempotence and Recovery

The searches, formatter, tests, and quality gates are safe to rerun. The code change is deletion-oriented and does not migrate data or alter files outside the repository. If a commit hook fails, keep the container running, fix the reported cause, rerun the narrow failing check, and retry the commit without bypassing hooks. If a review fix changes a previously reviewed area, rerun that lane or include it explicitly in the next required review wave.

The branch-local plan is intentionally committed while work is active so another contributor can resume from it. Remove only this plan file after all implementation and reviews complete; keep `docs/wip/AGENTS.md` intact.

### Artifacts and Notes

The issue-defined zero-result machine summary is:

    {"complete":true,"returned":0,"limit":50,"total":0}

The human empty text reuses current messages without warning decoration, for example:

    no signals found in selected scope

The diagnostic codes that remain fixed are:

    WPK-W0002  truncated output
    WPK-W0004  unmatched extraction candidate
    WPK-W0005  ambiguous signals omitted

### Interfaces and Dependencies

No new dependency, module, public Rust type, abstraction, configuration, or schema is required. `crate::engine::ResultSummary`, `crate::engine::CommandResult`, `crate::diagnostic::Diagnostic`, and the existing `crate::output` renderers remain the interfaces. `WarningDiagnosticCode` retains only currently emitted warning meanings with explicit code strings. Existing command data variants remain unchanged.

Revision note (2026-08-15 04:58Z): Created the initial self-contained execution plan after repository exploration and maintainer confirmation that `docs search` is out of scope.

Revision note (2026-08-15 05:13Z): Recorded completed implementation, tests, documentation, full CI evidence, and the minimal centralized human-rendering decision before peer review.

Revision note (2026-08-15 05:19Z): Recorded the completed Luna Max review wave and its resolved documentation and help findings.

Revision note (2026-08-15 05:26Z): Recorded the completed Terra High review wave and its resolved contract, regression-test, and extraction-simplification findings.

Revision note (2026-08-15 05:35Z): Recorded the Sol High control findings, their JSONL and benchmark-fixture fixes, auxiliary test evidence, and the clean control re-review.

Revision note (2026-08-15 05:37Z): Recorded the final post-review CI audit, coverage and FSDB evidence, completed outcomes, and remaining branch cleanup and PR steps.
