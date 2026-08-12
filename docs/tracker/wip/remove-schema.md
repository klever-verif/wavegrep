# Remove current JSON Schema support

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

WavePeek currently carries a large JSON Schema subsystem that users do not rely on. After this work, version 3 users can continue to consume deterministic `--json` envelopes, `--jsonl` streams, and structured extraction input, but WavePeek no longer generates, exports, validates, publishes, or advertises current schemas. JSON envelopes and JSONL begin records no longer contain `$schema`, structured input no longer requires it, and an old `$schema` property is ignored so otherwise compatible input keeps working.

A user can observe the result by running `wavepeek --help` and seeing no `schema` command, running representative JSON and JSONL commands and seeing no `$schema` property, and running every structured extraction family from an input document without `$schema`. Repository checks and a synthetic version 3 documentation publication must complete without generating a version 3 schema while preserving files already present on the Pages branch.

## Non-Goals

The bundled skill redesign, recipe-first rewrite, and tested standard-library Python post-processing example belong to issues #77 and #79 and are deferred by maintainer decision. This work does not redesign retained command payloads, JSON envelope fields other than `$schema`, JSONL ordering, diagnostics, or structured extraction semantics. It does not edit immutable historical release notes or delete already-published v1 and v2 files from the `gh-pages` branch.

## Progress

- [x] (2026-08-12 16:42Z) Read issue #89, repository guidance, quality and release documentation, and the execution-plan policy.
- [x] (2026-08-12 16:42Z) Installed worktree-local hooks, started the worktree container, prepared waveform fixtures, and established passing baselines for `schema_cli` (40 tests) and `jsonl_cli` (10 tests).
- [x] (2026-08-12 16:42Z) Mapped schema production, runtime, input, test, automation, documentation, and Pages-publication paths; verified that historical schemas survive because publication stages and exports the existing complete `gh-pages` tree.
- [x] (2026-08-12 16:48Z) Made all six structured extraction families independent of `$schema`, preserved unknown legacy-field compatibility through Serde, updated benchmark inputs, and passed 95 focused extraction tests, 93 benchmark helper tests, and Clippy.
- [x] (2026-08-12 16:58Z) Removed the schema CLI, engine, builders, artifacts, dependencies, and validator-only tests while retaining direct JSON/JSONL shape and sequencing checks; `cargo check --all-targets` and 14 focused integration suites passed.
- [x] (2026-08-12 17:05Z) Removed generation gates and current schema publication/deployment checks; 51 docs-helper tests include a synthetic v3 staging case that preserves a historical v2.2 file and creates no v3 schema, and `just test-aux` plus `just check-actions` passed.
- [ ] Replace public schema discovery with concise machine-output forms and examples; update maintainer docs, README, breadcrumbs, and changelog; run docs checks and commit the slice.
- [ ] Run `just ci`, conduct parallel correctness, docs, and mandatory KISS/YAGNI/ponytail challenge reviews, fix findings, rerun affected checks, and obtain a clean independent control review.
- [ ] Remove this branch-local plan, run `just check`, commit cleanup, push the branch, and open a pull request targeting `dev3`.

## Surprises & Discoveries

- Observation: The focused baseline tests initially failed only because ignored generated waveform fixtures did not exist in the new worktree.
  Evidence: `schema_cli` reported `cannot open 'tests/fixtures/generated/m2_core.vcd'`; after `./dev just prepare-waveform-fixtures`, all 40 `schema_cli` and all 10 `jsonl_cli` tests passed.

- Observation: Existing Pages publication updates and exports the complete `gh-pages` tree, so old root schema files remain available without retaining a compatibility publisher in current source.
  Evidence: The new `test_stage_publication_preserves_historical_files` stages v3 installers over a Pages branch containing `schema-output-v2.2.json`, then verifies the historical bytes remain and no v3 schema path exists; all 51 docs-helper tests pass.

- Observation: Runtime structured-input types are separate from the schema-only input DTO tree and use Serde's default unknown-field behavior.
  Evidence: After deleting explicit schema fields and checks, all 95 extraction integration tests passed; `extract_generic_source_file_ignores_schema_url` and `extract_axi_source_ignores_legacy_schema_url` passed with arbitrary or old `$schema` properties.

## Decision Log

- Decision: Defer all bundled-skill work, including the Python post-processing recipe, to issues #77 and #79.
  Rationale: The maintainer explicitly selected that boundary before implementation.
  Date/Author: 2026-08-12 / pi

- Decision: Remove schema support rather than replace it with weaker generated definitions or a compatibility command.
  Rationale: Issue #89 identifies no observed use and explicitly requires removal of current generation and export paths. Deletion is the smallest solution and avoids preserving maintenance cost without a user requirement.
  Date/Author: 2026-08-12 / pi

- Decision: Preserve legacy structured input by relying on existing Serde unknown-field behavior, not by retaining a schema field or URL allow-list.
  Rationale: This directly meets the compatibility requirement with less code and no schema-specific runtime path.
  Date/Author: 2026-08-12 / pi

- Decision: Preserve published v1/v2 schema URLs through the existing complete-tree Pages publication model, without current-source copies or a historical-schema republisher.
  Rationale: Historical files already live on `gh-pages`; retaining or recreating them in current source would be redundant and could accidentally overwrite immutable artifacts.
  Date/Author: 2026-08-12 / pi

- Decision: Retain direct JSON and JSONL behavioral tests while deleting schema-validator assertions.
  Rationale: Serialization, field shapes, diagnostics, stream sequencing, and deterministic ordering remain public runtime contracts even though JSON Schema does not.
  Date/Author: 2026-08-12 / pi

## Outcomes & Retrospective

Milestones 1 through 3 are complete. Structured input executes without schema metadata; the core schema subsystem and obsolete gates are gone; publication now stages only documentation and installers while exporting the complete Pages tree. Direct machine-output tests and a synthetic historical-file preservation test cover retained behavior. Public and maintainer documentation remain for the next milestone.

## Context and Orientation

WavePeek is a Rust command-line waveform inspector. `src/cli/` parses command-line arguments and `src/engine/` dispatches commands. Runtime machine-output data transfer objects, abbreviated DTOs, live under `src/contract/`; a DTO is a serializable Rust structure used to carry command output or structured input. `src/output.rs` renders human output, one JSON envelope, or a JSON Lines stream where each line is an independent JSON record.

Current schema support crosses several layers. `src/cli/schema.rs` and `src/engine/schema.rs` implement `wavepeek schema`. `src/contract/schema.rs` and protocol-specific `*_schema.rs` files construct exact schema branches. `src/schema_contract.rs` embeds generated files from `schema/`. `schemars` derives schema definitions from retained DTOs, while the development-only `jsonschema` crate validates output in tests. `tools/schema-gen/`, `tools/schema/`, root `justfile` recipes, and `.pre-commit-config.yaml` generate and verify snapshots.

Structured extraction inputs use runtime `SourceFile` structures in `src/engine/extract.rs`, `ahb.rs`, `apb.rs`, `atb.rs`, `axi.rs`, and `axistream.rs`. These currently deserialize and explicitly validate a `$schema` URL. The schema-only mirror definitions in `src/contract/input.rs` are not used to execute commands. Removing the runtime field and checks lets Serde ignore old `$schema` fields while accepting documents without them.

Documentation sources under `docs/public/` are embedded into the binary and exported into the versioned documentation website. `tools/docs/publish_docs.py` stages documentation and installer artifacts into an existing `gh-pages` checkout; `tools/docs/workflow_docs.py` converts staging metadata into deployment-check arguments; `tools/docs/check_deploy.py` checks deployed endpoints. Current code additionally publishes root schema files and requires version-derived schema endpoints. That schema-specific logic must disappear, while complete-tree export must remain so old v1/v2 files survive.

Repository commands run inside the worktree-specific development container through root `./dev`. Focused tests use `./dev cargo test ...`; stable gates use `./dev just ...`. Host Git hooks invoke that already-running container, so commits must not bypass hooks.

## Open Questions

There are no blocking product questions. Exact retained test placement can follow the smallest existing suite ownership discovered during edits. The release itself will set the package to version 3 separately; this implementation removes independent schema-version instructions but does not perform an unrelated release version bump.

## Plan of Work

### Milestone 1: Remove structured-input schema coupling

Edit the six runtime `SourceFile` implementations under `src/engine/` to remove `$schema` fields, schema URL imports, URL allow-lists, and URL validation while retaining all validation for input kind, protocol profile, mappings, includes, uniqueness, and conflicting options. Remove `$schema` from the four committed benchmark input documents under `bench/e2e/inputs/`.

Adapt existing extraction integration tests instead of adding a new framework. Each extraction family must exercise a valid source without `$schema`; one representative source must carry an arbitrary old `$schema` and still work, proving the unknown-property compatibility contract. Remove tests whose only purpose is accepting or rejecting schema URL versions. Run all six extraction suites and relevant benchmark auxiliary tests. Commit this independently verifiable behavior before broader deletion.

### Milestone 2: Delete the core schema subsystem and retain machine output

Delete the schema CLI and engine modules, dispatch enum variants, schema-only contract builders and input DTOs, embedded schema artifacts, generated `schema/` tree, generator/checker tools, `schemars`, and `jsonschema`. Remove only schema derives, annotations, helper implementations, URL fields, and URL constants from retained runtime DTOs. Keep output conversions, waveform-kind validation, JSON envelope rendering, JSONL sinks and records, diagnostics, summaries, and ordering.

Delete dedicated schema command/generator tests. Move or rewrite the small representative JSON tests from `tests/schema_cli.rs` into existing command suites, where they directly assert common object/list output forms, envelope keys, diagnostics, and absence of `$schema`. Keep all `tests/jsonl_cli.rs` stream parsing and sequencing checks, removing only schema-validator setup and adding direct begin-record absence checks. In protocol suites, preserve concrete JSON shape/value assertions after removing external schema validation. Compile and run the affected JSON, JSONL, command, and protocol tests before committing.

### Milestone 3: Remove repository and publication machinery

Remove schema recipes and dependencies from `justfile`, the pre-commit schema hook, the schema-specific skip name in hook tests, and schema wording in the CI job title. Remove schema catalog loading, root artifact staging, metadata, path matching, and fallback checks from docs publication helpers. Keep versioned docs, `latest`, installer publication, branch fast-forward checks, bundle verification, Pages API checks, unrelated-path protection, and whole-tree Pages export.

Adapt helper tests to prove the reduced publication contract. Add or retain a minimal case that seeds a historical root schema file in a disposable Pages tree, stages a version 3 docs publication, verifies no v3 schema is created, and verifies that the historical file still exists unchanged in the exported Pages artifact. Run docs helper unit tests, hook helper tests, `just check-actions`, and docs publication checks before committing.

### Milestone 4: Replace schema discovery documentation

Delete the public schema command topic and remove schema links, invocations, URLs, and `$schema` properties from README, embedded public topics, maintainer docs, release instructions, and path-scoped breadcrumbs. Do not rewrite historical changelog sections; add one concise `Unreleased` removal entry.

Expand `docs/public/reference/machine-output.md` just enough to become the direct discovery surface requested by issue #89. Show short real JSON examples for a common object payload, list payload, event payload, transfer payload, and JSONL begin/item/diagnostic/end order. Keep examples aligned with actual retained DTO serialization and omit `$schema`. Update architecture and testing prose so runtime output stability is owned by direct tests rather than generated schema snapshots. Run embedded docs tests and strict docs-site checks before committing.

### Milestone 5: Validate, review, and publish the branch

Run `./dev just ci`. Launch read-only reviews in parallel for runtime correctness/tests, docs/publication behavior, and a mandatory KISS/YAGNI/ponytail challenge whose job is to identify retained schema remnants, unnecessary replacement code, speculative abstractions, or tests/docs that should simply be deleted. Apply substantive findings in the main session, rerun affected checks, and request impacted re-reviews. Then launch a fresh independent control review over `dev3...HEAD` and stop only when it reports no substantive findings or the two-pass review cap is reached with explicit remaining issues.

Update this plan throughout. At completion, record evidence and outcome, remove this WIP file as required by repository policy, run `./dev just check`, commit cleanup, push `dev3-89/remove-schema`, and open a GitHub pull request against `dev3` that summarizes scope, deferred skill work, tests, review lanes, and historical Pages preservation.

### Concrete Steps

Run all commands from repository root `/home/esynr3z/projects/wavepeek/.worktrees/wavepeek/dev3-89-remove-schema`.

The baseline setup and checks are:

    ./dev --install-hooks
    ./dev true
    ./dev just prepare-waveform-fixtures
    ./dev cargo test --test schema_cli --quiet
    ./dev cargo test --test jsonl_cli --quiet

Expected focused baseline:

    schema_cli: 40 passed; 0 failed
    jsonl_cli: 10 passed; 0 failed

After Milestone 1, run:

    ./dev cargo test --test extract_generic_cli --test extract_ahb_cli --test extract_apb_cli --test extract_atb_cli --test extract_axi_cli --test extract_axistream_cli
    ./dev python3 -B -m unittest bench.e2e.test_perf

After Milestone 2, run the narrowest affected suites identified by `cargo test --no-run`, including:

    ./dev cargo test --test jsonl_cli
    ./dev cargo test --test info_cli --test scope_cli --test signal_cli --test value_cli
    ./dev cargo check --all-targets

After Milestone 3, use the `justfile`-owned helper entrypoints where available:

    ./dev just test-aux
    ./dev just check-actions
    ./dev just docs-site-check

For historical Pages preservation, use disposable directories under repository-root `tmp/`; never modify or force-push the real `gh-pages` branch. Record blob hashes or byte comparisons showing each seeded historical file is unchanged and no v3 schema path exists.

After Milestone 4 and for final validation, run:

    ./dev just docs-site-check
    ./dev just ci
    ./dev just check

Expected final behavior:

    ./dev cargo run --quiet -- --help

The command list does not contain `schema`.

    ./dev cargo run --quiet -- info --waves tests/fixtures/generated/m2_core.vcd --json

The output is one object with `command`, `data`, and `diagnostics`, and no `$schema` key.

    ./dev cargo run --quiet -- change --waves tests/fixtures/generated/m2_core.vcd --signals top.clk --jsonl

Every non-empty line parses as JSON; the first record is `begin`, sequence numbers are contiguous, the last record is `end`, and no record has `$schema`.

### Validation and Acceptance

The CLI, binary, root tasks, pre-commit configuration, workflows, release helpers, and Pages publisher must expose no current schema generation or export path. A repository search may retain only immutable historical prose where necessary and the deferred bundled skill references explicitly excluded by maintainer decision; executable current paths must be absent.

Representative `--json` tests must directly assert the `command` / `data` / `diagnostics` envelope and object, list, event, and transfer payload shapes without `$schema`. Representative `--jsonl` tests must parse every line and assert begin-first, contiguous sequence numbers, item/diagnostic ordering, summary consistency, end-last, and no `$schema` in begin records. Existing deterministic serialization and command runtime tests must pass.

Every structured extraction input family must execute without a schema URL. At least one test must prove an old or arbitrary `$schema` property is ignored when the rest of the document is compatible. Invalid input kind/profile/mapping behavior must remain covered.

Strict public docs generation must pass and machine-output documentation must show the required common forms with short examples. A synthetic version 3 publication must create no v3 schema artifacts and must retain seeded v1/v2 root files byte-for-byte in the final Pages artifact. Existing v1/v2 public URLs are not changed by this branch.

The complete `just ci` and `just check` gates must pass. The correctness, docs/publication, and mandatory KISS/YAGNI/ponytail reviews plus the independent control pass must complete before the PR opens.

### Idempotence and Recovery

Fixture preparation, focused tests, helper tests, and quality gates are safe to repeat. Generated fixtures are ignored. All publication tests must use uniquely owned paths below `tmp/`; do not clean unrelated files there. If a milestone fails, inspect `git diff`, fix only that slice, rerun its focused command, and commit after it passes. Small milestone commits allow a faulty slice to be reverted without disturbing later work.

Do not edit the real `gh-pages` branch during implementation. A synthetic checkout may be deleted only when its path was created by this work. Do not bypass Git hooks. If a commit hook fails, keep the container running, fix the reported cause, rerun the relevant gate, and retry the commit normally.

### Artifacts and Notes

Baseline evidence:

    running 40 tests
    ........................................
    test result: ok. 40 passed; 0 failed

    running 10 tests
    ..........
    test result: ok. 10 passed; 0 failed

Historical Pages root schemas known at planning time:

    wavepeek_v1.json
    wavepeek-stream-v1.json
    wavepeek_v2.0.json
    wavepeek-stream-v2.0.json
    schema-output-v2.1.json
    schema-stream-v2.1.json
    schema-input-v2.1.json
    schema-output-v2.2.json
    schema-stream-v2.2.json
    schema-input-v2.2.json

### Interfaces and Dependencies

At completion, there is no public `Schema` command variant, schema engine result, schema contract module, schema URL constant, or generated schema artifact. Runtime output continues to use Serde and `serde_json`; no replacement schema dependency or abstraction is introduced. `schemars` and development dependency `jsonschema` are absent from `Cargo.toml` and `Cargo.lock`.

`crate::contract::output::OutputEnvelope` retains serialized fields `command`, `data`, and `diagnostics`. `crate::contract::stream::BeginRecord` retains `type`, `seq`, and `command`. Other JSONL records retain their existing fields and order semantics. The six engine-local `SourceFile` types retain runtime extraction properties but no longer model or validate `$schema`.

Docs publication retains its current complete-tree export interface so existing `gh-pages` files survive. Staging metadata and deployment-check arguments no longer contain schema artifacts. No compatibility publisher, replacement discovery service, or schema-version configuration is added.

Plan revision note (2026-08-12): Created the initial self-contained plan after source/publication mapping and baseline verification. It records the maintainer-approved bundled-skill deferral and chooses deletion plus direct runtime tests as the minimal implementation.