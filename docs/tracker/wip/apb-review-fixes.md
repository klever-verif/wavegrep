# Tighten APB review contracts and examples

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with the `exec-plan` skill. All commands below run from `/workspaces/wavepeek/.worktrees/feat-extract-apb` on branch `feat/extract-apb` for pull request #69.

## Purpose / Big Picture

After these review fixes, the published JSONL schema will reject `begin` records whose command and context disagree, the APB input schema will reject empty mapping values before runtime, and the APB documentation examples will show every resolved mapping needed by their event payloads. A user or external validator can then rely on the generated schemas and examples to describe records that `wavepeek` can actually emit and execute.

The behavior is observable by validating three negative records: `extract apb` without APB context, `change` with APB context, and an APB source with a whitespace-only map value must all fail schema validation. Existing runtime-generated APB, AXI, and context-free JSONL records must continue to validate.

## Non-Goals

This work does not change APB event classification, waveform mapping, output rows, CLI flags, runtime JSONL serialization, schema version URLs, or existing AXI input-map validation. It does not add stream-wide validation for sequence continuity, stable commands, final summaries, or record ordering because `schema/stream.json` validates one record at a time.

## Progress

- [x] (2026-07-26 11:47Z) Reproduce and analyze all three review findings against runtime, generated schemas, tests, and docs.
- [x] (2026-07-26 11:47Z) Restore Git registration for the APB, AHB, ATB, and AXI-Stream worktrees and verify clean upstream-aligned states.
- [x] (2026-07-26 11:47Z) Create this branch-local ExecPlan for the accepted findings.
- [ ] Couple JSONL `begin` command and context in the generated stream schema and add positive and negative contract tests.
- [ ] Reject blank APB source map values in the input schema and add validator coverage.
- [ ] Correct both APB documentation mapping examples and strengthen embedded-doc tests.
- [ ] Regenerate schemas, run focused tests and `just check`, and commit coherent fixes.
- [ ] Perform a complete self-review, run `just ci`, record evidence, remove this WIP plan, and commit cleanup.
- [ ] Push `feat/extract-apb`, wait for PR #69 CI, and verify the updated remote PR state.

## Surprises & Discoveries

- Observation: Runtime JSONL output already enforces command/context pairing through separate `begin` and `begin_context` paths, but the shared `BeginRecord` DTO makes `context` optional and does not express command-dependent schema constraints.
  Evidence: `src/output.rs::JsonlWriter::begin_context` emits typed APB or AXI context, while generated `schema/stream.json::$defs.beginRecord` currently accepts any `streamContextData` or null for every command.

- Observation: The command/context weakness predates APB and also permits AXI context to be omitted or attached to unrelated commands.
  Evidence: `streamContextData` is an APB-or-AXI union independent of `beginRecord.command`.

## Decision Log

- Decision: Constrain `beginRecord` with three disjoint schema branches: APB with required APB context, AXI with required AXI context, and all other streaming commands with no `context` property.
  Rationale: This fixes the shared schema root cause, matches runtime serialization, keeps record-local validation separate from stream-wide invariants, and preserves outer-object extension behavior for unrelated property names.
  Date/Author: 2026-07-26 / pi

- Decision: Add a `\S` JSON Schema pattern to APB map values only.
  Rationale: Runtime trims map values and rejects empty results. The pattern rejects empty and whitespace-only strings while allowing values with surrounding whitespace that runtime normalizes. Tightening AXI input schemas is a separate existing-contract change and is outside this APB review slice.
  Date/Author: 2026-07-26 / pi

- Decision: Add the omitted `paddr`, `pwdata`, and `pslverr` mapping rows to both examples.
  Rationale: The examples intentionally demonstrate those payloads, and human output always lists every resolved mapping. Removing payloads would make the examples less useful.
  Date/Author: 2026-07-26 / pi

- Decision: Keep this ExecPlan committed during work and remove it before the updated PR is handed off.
  Rationale: `docs/tracker/wip/` is for restartable branch-local artifacts and its guidance requires cleanup before merge.
  Date/Author: 2026-07-26 / pi

## Outcomes & Retrospective

Implementation has not started. All three findings are confirmed: one medium public stream-schema gap and two low input-schema/documentation mismatches. No finding is a runtime APB extraction defect.

## Context and Orientation

`wavepeek` emits newline-delimited JSON through `src/output.rs::JsonlWriter`. Context-free commands call `begin`, which emits no context. APB and AXI adapters call `begin_context`, which serializes a protocol-specific context. `src/contract/stream.rs` defines `BeginRecord` with optional `StreamContextData`, an enum containing APB and AXI context variants. That optional Rust representation is needed by the serializer but is too broad as a generated public schema.

`src/contract/schema.rs` collects Schemars-generated definitions and then applies localized schema overrides. It already creates command-discriminated `item` record definitions. The minimal stream fix belongs here: retain the generated `beginRecord` object and add a `oneOf` constraint that correlates its required `command` and `context` fields. APB and AXI branches require their exact context reference. The context-free branch permits `info`, `scope`, `signal`, `value`, `change`, `property`, and `extract generic` and uses `not: { required: ["context"] }` so even null or protocol context is rejected.

`src/contract/apb_schema.rs::apb_source_input_schema` defines a generic `maps` property and `apb_input_maps_schema` narrows keys per APB profile and PREADY mode. Both currently constrain values only as strings. `src/engine/apb.rs::normalize_map_pair` trims each value and rejects it when empty, so generated `schema/input.json` must require at least one non-whitespace character using pattern `\S` at both APB map schema locations.

`tests/schema_cli.rs` compiles the checked-in schemas with the `jsonschema` crate. Its APB stream test verifies context internals but does not test command/context pairing. The fix must add positive context-free, APB, and AXI begin records and negative missing, null, wrong-protocol, and unrelated-command context cases. `tests/extract_apb_cli.rs::extract_apb_source_schema_accepts_canonical_values_only` is the focused place to reject empty and whitespace-only map values while retaining a nonblank value.

`docs/public/commands/extract.md` and `docs/public/workflows/extract-handshake.md` show APB human output. Their event rows contain `paddr`, `pwdata`, and `pslverr`, but their `mappings` sections omit those three resolved signals. `tests/docs_cli.rs` loads packaged Markdown through the actual docs command and can prevent this inconsistency from recurring.

Generated schema snapshots are `schema/input.json`, `schema/output.json`, `schema/stream.json`, and `schema/catalog.json`. They must be regenerated with `just update-schema`, never edited manually. `tools/schema/check_schema_contract.py` verifies durable schema structure and should assert the new begin command/context constraint and APB nonblank map-value constraint.

## Open Questions

There are no blocking open questions. The accepted findings and minimal root-cause fixes are fully specified.

## Plan of Work

First, update `src/contract/schema.rs` after stream definitions are generated. Add one localized helper that inserts a disjoint command/context `oneOf` into `beginRecord`. Reuse the existing APB and AXI context definitions and enumerate only commands that already appear in `stream_commands()`. Extend `tests/schema_cli.rs` with records proving every branch and rejection case. Extend `tools/schema/check_schema_contract.py` so schema deployment checks cannot silently lose this constraint.

Second, add a small APB nonblank-string schema helper in `src/contract/apb_schema.rs` and use it for the generic APB map-value rule and every profile-specific map property. Extend APB input validator tests with empty and whitespace-only values. Do not alter runtime normalization or AXI schema branches.

Third, add the three missing mapping rows to each public APB example and assert those exact rows through `tests/docs_cli.rs`. Regenerate schema snapshots, run focused schema and docs tests, then run `just check`.

Finally, inspect the full branch diff and each accepted finding, verify no unrelated public behavior changed, run `just ci`, and record the evidence here. Commit the completed plan, remove it in a cleanup commit, push the branch, and watch PR #69 checks to completion.

### Concrete Steps

Run from `/workspaces/wavepeek/.worktrees/feat-extract-apb`:

    cargo fmt --all
    just update-schema
    just check-schema
    cargo test --test schema_cli
    cargo test --test extract_apb_cli extract_apb_source_schema_accepts_canonical_values_only
    cargo test --test docs_cli public_extract_docs_cover_apb_profiles_modes_and_stateless_scope
    just check
    just ci

Expected focused evidence is:

    test result: ok. <N> passed; 0 failed
    schema contract OK

Delivery commands are:

    git push origin feat/extract-apb
    gh pr checks 69 --repo kleverhq/wavepeek --watch --interval 10
    gh pr view 69 --repo kleverhq/wavepeek --json url,state,mergeable,statusCheckRollup

### Validation and Acceptance

A valid context-free `change` begin record without `context`, a valid APB begin with `extractApbContext`, and a valid AXI begin with `extractAxiContext` must validate. APB and AXI begins without context, either protocol command with the wrong protocol context, and context-free commands with any `context` property must fail.

Canonical APB source JSON with a nonblank mapped waveform name must validate. Empty and whitespace-only APB map values must fail. Runtime source parsing behavior must remain unchanged.

Both APB docs topics must list `paddr`, `pwdata`, and `pslverr` in the mapping section whenever their example event rows contain those payloads. Existing embedded docs tests must remain green.

`just check` and `just ci` must exit successfully. The updated PR must remain open against `main`, retain `Closes #66`, and have successful remote checks.

### Idempotence and Recovery

Formatting, schema generation, tests, and repository gates are idempotent. Schema snapshots must be regenerated, not hand-edited. If a hook or gate fails, fix the reported problem and commit normally without bypassing hooks. The worktree registration is restored and must not be removed or pruned. If push or remote checks fail, keep local commits, diagnose the failure, retry, and verify branch synchronization before handoff.

### Artifacts and Notes

The accepted review findings were supplied from a local review of PR #69. No external service or new dependency is required. Disposable logs belong under repository-root `tmp/`.

At completion, replace this paragraph with concise commit, gate, review, and PR evidence before removing the plan; its committed history will retain the final state.

### Interfaces and Dependencies

No runtime interface or dependency changes are required. `src/contract/schema.rs` will gain one private schema-generation helper equivalent to:

    fn apply_stream_begin_context_constraints(defs: &mut Map<String, Value>);

It must mutate only `$defs.beginRecord` and add command-discriminated context requirements. `src/contract/apb_schema.rs` will gain a private helper equivalent to:

    fn nonblank_string_schema() -> Value;

It must return a JSON Schema with `type: "string"` and `pattern: "\\S"` and be used only for APB source map values.

Revision note (2026-07-26): Initial plan created after reproducing all review findings and restoring the neighboring AMBA worktree registrations.
