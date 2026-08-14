# Add flat raw-value bit-range projections

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with the repository's `exec-plan` skill. It is a branch-local artifact and will be removed after implementation and review, before the pull request is opened.

## Purpose / Big Picture

WavePeek users need to inspect a small static range of a wide dumped vector without receiving the complete value. After this change, `value --signals`, `change --signals`, and `extract generic --payload` accept a trailing `[msb:lsb]`, sample the existing flat integral waveform leaf, and emit only that normalized bit range. A user can run `wavepeek value --signals 'expanded_key_ff[255:128]'` and observe a projected path with a 128-bit Verilog literal in human, JSON, or JSONL output.

The implementation must remain backend-neutral. VCD, FST, and optional FSDB backends continue resolving and sampling the complete source leaf; the engine projects the normalized sampled bit string before comparison and rendering.

## Non-Goals

This change does not add `[n]` bit selection, dynamic or indexed ranges, chained or multidimensional selections, expression-language selection, protocol payload mapping selection, hierarchy reconstruction, packed-dimension awareness, unpacked arrays, struct fields, or an extensible public selector schema. `[n:n]` is the only one-bit projection syntax. An exact waveform path always wins before projection parsing.

## Progress

- [x] (2026-08-14 17:40Z) Read issue #94, repository guidance, public contracts, command engines, waveform resolution and sampling, tests, fixture policy, and review requirements.
- [x] (2026-08-14 17:40Z) Resolve product questions: source-file generic payloads support projections; request order and duplicates are preserved in every command, including `extract generic`.
- [x] (2026-08-14 18:08Z) Add focused tests proving parsing, normalized slicing, scoped names, output shape, duplicate/overlapping/full selections, invalid diagnostics, JSONL, and VCD/FST/FSDB parity.
- [x] (2026-08-14 18:08Z) Implement one shared engine-level flat projection resolver and sampler, then integrate `value`, `change`, and `extract generic` without backend changes.
- [x] (2026-08-14 18:08Z) Make every applicable `change` execution path compare projected values for sparse, delta, and wildcard decisions.
- [x] (2026-08-14 18:08Z) Update CLI help, public packaged-skill references, and contract tests.
- [x] (2026-08-14 18:10Z) Run focused tests, VCD/FST parity, FSDB tests, `just ci`, and `just check`; all passed.
- [x] (2026-08-14 18:33Z) Run Luna Max correctness/tests, architecture/performance, and docs/contracts review lanes; fix the fused first-timestamp wildcard bug, reduce hot-loop cloning/tracking, expand optimized-engine projection tests, and correct docs/help findings.
- [x] (2026-08-14 18:41Z) Run Terra High over the same three lanes; correctness was clean, while architecture/docs findings removed fused full-width projection clones and completed change one-bit help.
- [ ] Run independent Sol High control review, apply any final findings, and rerun gates.
- [ ] Remove this WIP plan, commit the reviewed result, push the branch, and open a pull request closing issue #94.

## Surprises & Discoveries

- Observation: `value` already promises duplicate preservation and waveform sampling has a duplicate-preserving projection helper, while `extract generic` explicitly rejects duplicate payload paths before and after scoped resolution.
  Evidence: `skills/wavepeek/references/command-model.md`, `src/waveform/mod.rs::sample_signals_at_time`, and `src/engine/extract.rs::require_unique_payloads`.

- Observation: `change` has baseline, fused, edge-fast, and pre-edge paths. Sparse and delta decisions converge at `emit_row`, but wildcard event matching occurs before that function and currently compares complete source values.
  Evidence: `src/engine/change.rs::{run_baseline_emit,run_fused_emit,run_edge_fast_emit,run_pre_edge_emit,emit_row}` and `src/expr/eval.rs::any_tracked_matches`.

- Observation: all backends already return normalized sampled bit strings, so bit zero maps to the rightmost byte without declaration-direction metadata.
  Evidence: `src/waveform/types.rs::SampledSignalState`, Wellen `bit_string()` conversion, and FSDB sampled-width validation.

- Observation: the existing sparse/delta contract emits every requested position in the first selected row, then only changed positions. Projecting before `emit_row` preserved that behavior without special cases.
  Evidence: `tests/change_cli.rs::change_compares_projected_values_for_wildcard_sparse_and_delta_rows` passes in forced baseline and fused modes.

- Observation: VCD/FST and FSDB preserve known bits around unknown values sufficiently for range slicing; literal formatting may collapse an unknown-containing selected value after slicing, as intended.
  Evidence: `value_projects_normalized_ascending_and_unknown_bits` and `fsdb_value_json_matches_vcd_sampling_contract` pass.

- Observation: Luna Max found fused wildcard evaluation treated a first dump timestamp as a change even when no previous timestamp existed, unlike baseline evaluation.
  Evidence: gating the projected comparison on `previous_timestamp.is_some()` makes the forced fused and baseline dense wildcard regression emit only the real in-range change at 10ns.

- Observation: Terra High found fused wildcard preparation still cloned complete rolling strings before narrow slicing.
  Evidence: `project_rolling_samples` now borrows rolling source bits and allocates only the selected output; fused/baseline and optimized dispatch projection tests pass.

## Decision Log

- Decision: Require `[n:n]` for one bit and leave `[n]` exclusively as waveform path syntax.
  Rationale: exact-first fallback for `[n]` would make meaning depend on dump contents and could silently select a real indexed leaf. Help and diagnostics will show `[n:n]` explicitly.
  Date/Author: 2026-08-14 / user and coding agent.

- Decision: Keep serialized `path` and `relative_path` as the selected path including `[msb:lsb]`; do not add `source_path`, `projection`, or a third path field.
  Rationale: issue #94 specifies this shape, arrays already preserve order and duplicates, and a speculative public selector schema would add contract surface without a present consumer.
  Date/Author: 2026-08-14 / user and coding agent.

- Decision: Keep source resolution and projection separate internally.
  Rationale: backends and candidate collectors require the real canonical source path, while output and comparisons require the projected path and width.
  Date/Author: 2026-08-14 / coding agent.

- Decision: Remove `extract generic` payload uniqueness rejection and preserve every request position.
  Rationale: issue #94 explicitly requires duplicates across the in-scope raw-value commands, and the user confirmed duplicate payloads must be allowed.
  Date/Author: 2026-08-14 / user.

- Decision: Support projections in generic extractor JSON source files through the same payload resolver used by CLI `--payload`.
  Rationale: the user confirmed source-file support, and both inputs already share one execution plan.
  Date/Author: 2026-08-14 / user.

- Decision: Consume sampled states during projection and deduplicate fused tracking IDs, but do not add extract-row remapping or bulk projection resolution.
  Rationale: consuming removes full-width clones and fused deduplication deletes unused state with small local changes. Extract remapping and batch fallback add code without measured need; payload lists are small and correctness gates already pass.
  Date/Author: 2026-08-14 / coding agent after Luna Max review.

## Outcomes & Retrospective

Implementation and user documentation are complete. One shared engine module resolves and slices selections, all three command surfaces preserve duplicate positions, and projected wildcard comparison is covered in baseline, fused, VCD/FST, and FSDB tests. `just ci` passes, including 665 unit tests, integration suites, 22 FSDB CLI tests, documentation publishing checks, and coverage above 92%. Reviews, `just check`, plan cleanup, and the pull request remain.

## Context and Orientation

WavePeek is a Rust CLI. `src/cli/` parses flags and owns detailed help. `src/engine/` executes commands. `src/waveform/` resolves canonical dump paths and samples backend-neutral values. `src/contract/` and `src/output.rs` serialize and render engine rows. The packaged user documentation under `skills/wavepeek/` is embedded into the binary and is the public source of truth.

A canonical path is the full dot-separated waveform leaf name, such as `top.cpu.data`. A relative path is the same leaf shown relative to `--scope`, such as `data`. A projection is the new trailing static range, such as `[7:4]`. It indexes the normalized sampled string: bit zero is the rightmost least-significant character, so a width-`W` source range `[msb:lsb]` uses byte offsets `W - 1 - msb .. W - lsb`.

`src/engine/value.rs::resolve_requested_signals` currently scopes complete tokens, resolves them in bulk, and samples complete leaves. `src/engine/change.rs::RequestedSignal` stores display and output paths while separate `ResolvedSignal` values drive candidate collection and several execution engines. `src/engine/extract.rs::resolve_payload_signals` resolves complete payload paths and currently rejects duplicates. `src/waveform/types.rs::ResolvedSignal` stores canonical source path, backend signal ID, and width. `SampledSignalState` stores a source path, width, and optional normalized bit string.

Exact path precedence matters because waveform leaves may legally contain brackets. Resolution must first try the complete scoped token. Only when it does not resolve may one trailing colon range be removed and the remaining base passed through the existing diagnostic resolver. A complete exact leaf such as `top.steps[0]` or `top.named[7:4]` remains unchanged.

`change --on '*'` means any requested value changed. Candidate timestamps may still be collected from complete source leaves; this can create harmless extra candidates. Event acceptance, sparse rows, and delta entries must compare projected strings so changes outside all requested ranges do not count. Explicit named, `posedge`, `negedge`, and `edge` terms retain complete-signal expression semantics.

## Open Questions

No product questions remain. If implementation reveals an exact bracketed-name ambiguity that existing typed errors cannot distinguish from absence, retain the issue's literal rule: only a successfully resolved complete token wins; otherwise parse at most one trailing projection.

## Plan of Work

First add a compact source-backed waveform fixture or minimally extend `tests/fixtures/source/value_vectors.v` so one normalized vector changes outside and then inside a selected range. Generate VCD and FST through the existing fixture recipe. Use a tiny inline VCD only for an exact leaf whose name contains brackets if ordinary HDL generation cannot preserve that spelling. Add integration assertions to `tests/value_cli.rs`, `tests/change_cli.rs`, and `tests/extract_generic_cli.rs`; add JSONL coverage in `tests/jsonl_cli.rs`, VCD/FST parity in existing parity suites, and FSDB coverage in `tests/fsdb_cli.rs`. Tests must include `[n:n]`, scoped and canonical names, request order, exact duplicates, overlapping projections, a projection plus its full source, x/z preservation, ascending declaration normalization, malformed/reversed/out-of-range bounds, source-file payloads, and exact path precedence. Keep each contract represented by the smallest test that proves it rather than creating a matrix for every equivalent output mode.

Next add one small shared engine module, tentatively `src/engine/signal_projection.rs`. Define a concrete type for an optional flat bit range and a resolved selected signal containing the untouched `ResolvedSignal`, selected output path, selected width, and optional range. The resolver accepts a waveform, query tokens, and optional scope; it preserves list order and duplicates, tries each complete canonical token first, parses only one trailing decimal `[msb:lsb]` after exact failure, resolves the base with existing scoped diagnostics, validates `msb >= lsb` and `msb < source.width`, and returns clear `WavepeekError::Signal` diagnostics. The sample helper slices only ASCII normalized bits with checked offsets, preserves missing samples, and returns the selected path and width. Do not add dependencies or change waveform backends.

Integrate the shared type into `src/engine/value.rs`: resolve selected signals, sample their source `ResolvedSignal` values, project them, then reuse `format_verilog_literal` and existing output rows. Integrate it into `src/engine/extract.rs`: remove both payload uniqueness checks, resolve every payload position independently, sample source leaves in `build_row`, and project before formatting. This automatically covers CLI and JSON source-file payloads.

Integrate it into all `src/engine/change.rs` paths. Baseline and optimized samples must be projected before initializing or updating `previous_values`, `changed_values_and_update`, and `build_snapshot`. For wildcard event terms, add the smallest concrete evaluator hook that lets `change` supply whether any selected projected value changed while existing callers and explicit event terms retain current semantics. Baseline computes this from projected samples at the candidate and preceding waveform timestamps. Fused mode computes it from rolling current and previous source bits after applying each requested range. Edge-fast and pre-edge retain their existing restrictions/fallbacks. Candidate source IDs remain complete and may be deduplicated only as an internal optimization; requested output positions are never deduplicated.

Update detailed clap help and argument descriptions in `src/cli/value.rs`, `src/cli/change.rs`, `src/cli/extract.rs`, and `src/cli/mod.rs`. Update `skills/wavepeek/references/value.md`, `change.md`, `extract.md`, `command-model.md`, `scoped-vs-canonical-names.md`, and `machine-output.md` as needed. State that this is a flat normalized-value projection, show `[n:n]`, exact-path precedence, output path behavior, invalid forms, duplicate preservation, and `change` projected comparison. Update `tests/cli_contract.rs` with durable help fragments.

Commit the plan, tests/implementation, and docs in coherent conventional commits. After gates pass, run read-only focused review lanes for correctness/tests, architecture/performance, and docs/contracts. The first parallel wave uses Luna Max; the second parallel wave uses Terra High over the same lane definitions. Every reviewer must apply KISS, YAGNI, and ponytail-review and return severity plus file/line findings or `No substantive findings`. Apply valid findings in the main session and rerun affected tests after each wave. A fresh Sol High reviewer then performs the independent consolidated control pass. Remove this plan after recording final evidence and outcomes, commit cleanup and review fixes, push, and open a GitHub pull request that closes #94.

### Concrete Steps

Run all commands from the repository root `/home/esynr3z/projects/wavepeek/.worktrees/wavepeek/dev3-94`. The host owns Git and GitHub operations; Rust, waveform tools, hooks, and gates run through `./dev`.

Start the worktree container and install hooks, which is idempotent:

    ./dev true
    ./dev --install-hooks

Generate source-backed waveform fixtures after fixture edits:

    ./dev just prepare-waveform-fixtures

Run narrow tests while implementing, for example:

    ./dev cargo test --test value_cli
    ./dev cargo test --test change_cli
    ./dev cargo test --test extract_generic_cli
    ./dev cargo test --test jsonl_cli
    ./dev cargo test --test value_vcd_fst_parity --test change_vcd_fst_parity --test extract_generic_vcd_fst_parity

Run optional FSDB coverage through its supported recipe:

    ./dev just test-fsdb

Before reviews, run the complete behavior gate and local handoff gate:

    ./dev just ci
    ./dev just check

Expected successful recipes exit zero. FSDB gates either pass when the mounted SDK is usable or print the repository's explicit supported skip result when unavailable.

Inspect and commit on the host without bypassing hooks:

    git status --short
    git diff --check
    git commit -m "feat(raw): add flat bit-range projections"

After reviews and cleanup, push and open the PR:

    git push -u origin dev3-94
    gh pr create --repo kleverhq/wavepeek --base main --head dev3-94 --title "feat(raw): add flat bit-range projections" --body-file tmp/issue94/pr-body.md

### Validation and Acceptance

A projected `value` query must emit the selected path and selected-width literal in human and JSON output. `[0:0]` must emit width one. Exact `steps[0]` and exact bracketed leaf paths must resolve as leaves, not projections. Scoped relative input must emit canonical selected `path` and selected `relative_path`.

Invalid negative, dynamic, malformed, reversed, chained, and out-of-range selections must exit one before sampling with a clear signal diagnostic. `[n]` must continue through ordinary waveform path resolution and never become a bit projection.

For `change`, a complete-source change outside `[msb:lsb]` must not emit a sparse row, appear in delta, or satisfy wildcard selected-value sampling. A change inside must emit. Dense/full rows must always contain selected values; dense/delta rows may contain an empty signal array. Overlapping ranges, duplicate ranges, and a range plus its full source must remain separate ordered positions.

`extract generic` must accept projections from both CLI `--payload` and JSON source files, preserve duplicate entries, and render projected paths and literals. Protocol-specific extractors remain unchanged.

Equivalent VCD and FST commands must produce equal payloads. FSDB tests must compare projected FSDB results with the generated VCD fixture for all three command surfaces. `./dev just ci` and `./dev just check` must exit zero after the final reviewed diff.

### Idempotence and Recovery

Fixture generation, formatting, tests, and hook installation are safe to repeat. Generated VCD/FST and FSDB files are ignored; do not add them to Git. Keep scratch output under `tmp/issue94/` and do not delete unrelated `tmp/` content. If a commit hook fails, fix the reported issue and commit normally; never bypass hooks. If a reviewer fails to return usable output, restart that required lane with the same model and focus rather than treating it as a pass.

### Artifacts and Notes

Issue #94 fixes this serialized shape:

    {"path":"top.expanded_key_ff[255:128]","value":"128'h..."}

For source width `W`, projection `[msb:lsb]` maps normalized bytes as:

    start = W - 1 - msb
    end_exclusive = W - lsb

The selected width is `msb - lsb + 1`. Since normalized values contain one ASCII character per bit, checked Rust string slicing at these offsets is valid after backend width validation.

### Interfaces and Dependencies

Use only the Rust standard library and existing repository dependencies. The shared engine projection type must retain the untouched `crate::waveform::ResolvedSignal` for backend calls and expose selected output path, selected width, and checked sample projection. Its resolver must call `crate::engine::scoped_signal_path`, `Waveform::resolve_signals`, and the existing diagnostic resolution path rather than duplicating hierarchy traversal.

Keep `crate::waveform::{ResolvedSignal,SampledSignalState}` and all backend public behavior unchanged. Keep the existing JSON DTO field names unchanged. If wildcard evaluation needs an override, preserve the current public `event_matches_at` entrypoint and make existing non-`change` callers receive identical semantics.

Plan revision note (2026-08-14): Initial self-contained plan created after repository research and user decisions on `[n:n]`, JSON shape, source-file payload support, and duplicate preservation.

Plan revision note (2026-08-14 18:08Z): Recorded completed implementation, docs, cross-backend tests, wildcard behavior, and passing `just ci`; review and handoff work remains.

Plan revision note (2026-08-14 18:33Z): Recorded passing `just check`, Luna Max review findings, the fused wildcard fix, lean performance changes, expanded engine coverage, and rejected speculative optimizations.

Plan revision note (2026-08-14 18:41Z): Recorded clean Terra correctness review, borrowed fused projection, final one-bit help wording, and focused verification.
