# Add actionable missing-signal diagnostics

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

WavePeek currently reports every failed signal lookup as “not found in dump,” even when the dump contains a copyable signal name elsewhere in the selected namespace. After this change, `value`, `change`, `property`, and every `extract` path will suggest a short deterministic list for wrong paths and close spelling mistakes. When no useful candidate exists, the error will instead explain that the signal may have been optimized, aliased, or omitted from the dump. Existing expression errors will retain their `fatal: expr` shape, and all failures will retain exit code 1.

A user can observe the change with `tests/fixtures/generated/m2_core.vcd`: querying `valid` under `--scope top` will suggest `cpu.valid`, while querying `missing` will explain that no dumped signal has that basename. Equivalent VCD, FST, and optional FSDB lookups will use the same backend-neutral diagnostic construction.

## Non-Goals

This work does not add automatic signal selection, fuzzy success, configurable matching policy, new CLI flags, or a new dependency. It does not change successful name resolution, expression syntax, JSON result schemas, or FSDB ambiguity handling. It does not redesign the string-backed public error enum beyond what is necessary to preserve existing error categories.

## Progress

- [x] (2026-08-14 04:48Z) Read issue #84, related issue #80, the linked practical-usage report, repository guidance, architecture, lookup implementations, and relevant test contracts.
- [x] (2026-08-14 04:48Z) Confirm product choices with the maintainer: at most five suggestions ranked by exact basename, edit distance, and lexical order; expression failures retain the existing `fatal: expr` presentation.
- [x] (2026-08-14 05:02Z) Implement one backend-neutral missing-signal diagnostic and route direct, expression, payload, and protocol mapping lookups through it.
- [x] (2026-08-14 05:02Z) Add focused CLI tests for exact basename, close spelling, absent signals, naming mode, bounded ordering, VCD/FST parity, expression envelopes, payloads, protocol mappings, and FSDB parity; preserve ambiguity through existing FSDB quarantine coverage.
- [x] (2026-08-14 05:02Z) Update packaged command and machine-output contracts, correct the empty-result guide, and add the changelog entry.
- [x] (2026-08-14 05:02Z) Run focused tests and `./dev just ci`; all mandatory, coverage, docs, and FSDB gates passed. Implementation commit remains.
- [ ] Run parallel Luna Max reviews for correctness/tests, documentation/contracts, architecture/KISS, and performance; fix findings and revalidate.
- [ ] Run parallel Terra High reviews over the same four areas; fix findings and revalidate.
- [ ] Run one independent Sol High control review, fix any substantive findings, and run the final quality gate.
- [ ] Remove this branch-local plan, commit cleanup, push the branch, and open a PR that closes issue #84.

## Surprises & Discoveries

- Observation: backend ambiguity cannot safely be inferred from the `WavepeekError::Signal(String)` variant.
  Evidence: `src/waveform/fsdb_hierarchy.rs` separately records `ambiguous_signal_by_path`, and recursive `SignalListing` values expose those paths through `omitted_ambiguous_paths`.

- Observation: direct lookups and expression lookups share the waveform facade but require different resolution capabilities.
  Evidence: direct values use `ResolvedSignal`, while real, string, enum, and event expression references use `ExprResolvedSignal`; suggestions must be checked through the same resolver as the failed query.

- Observation: `value` defers resolution until sampling, after its original scoped display names have been separated from canonical paths.
  Evidence: `src/engine/value.rs::resolve_requested_signals` only constructs names, and `Waveform::sample_signals_at_time` later calls raw `resolve_signals`.

- Observation: protocol mappings with `--scope` currently accept only local leaf names, unlike recursive generic scoped lookups.
  Evidence: each protocol `explicit_mappings` function rejects names containing `.`. The contextual protocol resolver therefore searches only the selected scope so every suggestion is accepted by the current mapping parser.

- Observation: the full quality gate can exercise the optional backend in this environment.
  Evidence: `./dev just ci` reported `ok: fsdb: Verdi FSDB Reader SDK found`, passed 666 library tests, 20 FSDB CLI tests, documentation checks, and 92.77% minimum source coverage before the focused FSDB suggestion test was added.

## Decision Log

- Decision: Keep at most five suggestions, rank exact basename matches before close spelling matches, then by edit distance and displayed path.
  Rationale: This is bounded and deterministic, puts the strongest evidence first, and follows the maintainer-approved default without exposing configuration.
  Date/Author: 2026-08-14, maintainer and coding agent.

- Decision: Preserve expression diagnostics as `fatal: expr` and place the shared missing-signal text inside the existing semantic diagnostic.
  Rationale: The maintainer explicitly chose the existing presentation; parser spans and expression error codes remain useful and stable.
  Date/Author: 2026-08-14, maintainer.

- Decision: Discover candidates from existing recursive hierarchy listings and filter them by the resolver used by the failed surface.
  Rationale: Listings are already deterministic and backend-neutral, FSDB listings identify ambiguous paths, and resolver filtering avoids suggesting entries that the caller still cannot use.
  Date/Author: 2026-08-14, coding agent.

- Decision: Implement edit distance locally without adding a dependency.
  Rationale: The standard library has no edit-distance function, but the required bounded comparison is small and only runs after a failed lookup; a dependency would exceed the need.
  Date/Author: 2026-08-14, coding agent.

- Decision: Limit scoped protocol mapping candidates to direct members of the selected scope.
  Rationale: Protocol mapping validation currently rejects dotted relative names. Suggesting recursive descendants would violate the issue requirement that every suggestion be valid in the active naming mode; generic and expression lookups remain recursive.
  Date/Author: 2026-08-14, coding agent.

## Outcomes & Retrospective

The implementation and first complete quality gate are finished. One shared waveform-facade policy now produces bounded candidates for direct and expression resolution, with thin engine call-site changes and documented fatal-output behavior. Review waves, any resulting fixes, final WIP cleanup, and PR publication remain.

## Context and Orientation

WavePeek is a Rust command-line waveform inspector. `src/engine/` converts CLI arguments into canonical signal paths. A canonical path is the full dot-separated hierarchy name, such as `top.cpu.valid`; with `--scope top`, the copyable relative query name is `cpu.valid`. `src/waveform/mod.rs` is the backend-neutral facade over VCD/FST support in `src/waveform/wellen_backend.rs` and optional FSDB support in `src/waveform/fsdb_backend.rs` and `src/waveform/fsdb_hierarchy.rs`.

Direct signal lists eventually call `Waveform::resolve_signals`. Expression references pass through `src/engine/expr_runtime.rs::ScopedExprHost`, then `src/waveform/expr_host.rs::WaveformExprHost`, and finally `Waveform::resolve_expr_signal`. Protocol `--map` handling in `src/engine/ahb.rs`, `apb.rs`, `atb.rs`, `axi.rs`, and `axistream.rs` currently calls raw direct resolution. Generic payload lookup is in `src/engine/extract.rs`; direct `change` lookup is in `src/engine/change.rs`; `value` samples through `Waveform::sample_signals_at_time`.

`SignalListing` in `src/waveform/types.rs` contains resolvable hierarchy entries and a list of omitted ambiguous FSDB paths. Candidate discovery must use recursive entries under the selected scope. Without a scope it must enumerate the full hierarchy. A scoped displayed candidate removes the exact `scope.` prefix; an unscoped candidate remains canonical. The requested basename is the final component after the last dot.

A “close spelling” candidate has a small Levenshtein edit distance from the requested basename. Levenshtein distance counts single-character insertions, deletions, and substitutions. The implementation will accept distance 1 for names up to three characters and distance 2 for longer names. This is deliberately conservative so unrelated large dumps do not produce noisy suggestions.

## Open Questions

No product questions remain. If optional FSDB fixtures are unavailable locally, the shared hierarchy unit tests plus the repository’s conditional FSDB quality gate will provide code-level coverage, and the exact skipped gate will be recorded.

## Plan of Work

First, add private candidate-discovery and formatting logic to `src/waveform/mod.rs`. It will enumerate recursive signal entries in the active scope or all top-level scopes, compare candidate basenames, sort exact basename matches before close matches, verify each candidate with either direct or expression resolution, omit ambiguous or unusable entries, and return at most five displayed names. The helper will preserve a backend error when the requested canonical path is listed or explicitly quarantined as ambiguous; only a genuinely absent canonical path receives the new diagnostic.

Expose the smallest contextual facade methods needed by engines: one bulk direct resolver that accepts canonical paths, the caller’s query names, and optional scope, plus one contextual expression resolver. Raw backend resolver methods remain available internally for sampling and candidate validation. `WaveformExprHost` will retain optional scope context so the existing `ScopedExprHost` can continue canonicalizing expression names while missing-expression errors use the same diagnostic body. Existing expression rendering remains unchanged.

Update `src/engine/value.rs`, `change.rs`, `extract.rs`, and protocol mapping functions to use contextual direct resolution. For `value`, perform contextual validation once before sampling because its current sampling helper no longer has the original scoped query spelling. Do not add command-specific suggestion implementations.

Add unit tests beside the waveform facade for ranking, bounds, relative/canonical display, edit-distance behavior, and unusable candidate filtering. Add or update CLI tests using the source-backed `m2_core` VCD/FST fixture to cover a wrong path with `cpu.valid`, a close typo, and an absent signal. Exercise direct, expression, and payload surfaces across the command families without multiplying equivalent test cases. Add FSDB hierarchy or optional CLI assertions for ambiguous-path exclusion and parity using existing fixtures rather than adding binary dumps.

Update `skills/wavepeek/references/command-model.md` with resolution-failure behavior and `skills/wavepeek/references/machine-output.md` with multiline fatal diagnostics. Add a concise unreleased changelog entry. No architecture update is needed unless implementation changes module ownership beyond the existing waveform facade.

Commit logical milestones after tests pass. Then run four read-only focused reviewers in parallel for each required review wave: correctness/tests, docs/contracts, architecture/KISS/YAGNI/ponytail, and performance. Luna Max is the first wave; Terra High repeats the same areas on the corrected diff. Apply fixes in the main session and rerun affected tests after each wave. Finally run one fresh Sol High read-only control review over the consolidated branch, resolve substantive findings, run `./dev just ci`, remove this WIP plan, commit, push, and open a PR against `dev3` that references and closes issue #84.

### Concrete Steps

All commands run from `/home/esynr3z/projects/wavepeek/.worktrees/wavepeek/84`.

Start and prepare the worktree container before commits:

    ./dev --install-hooks

Iterate with formatting and focused Rust tests:

    ./dev just format
    ./dev cargo test --test value_cli
    ./dev cargo test --test change_cli
    ./dev cargo test --test extract_generic_cli
    ./dev cargo test waveform::

Exercise the two visible outcomes:

    ./dev cargo run -- value --waves tests/fixtures/generated/m2_core.vcd --scope top --signals valid --at 5ns
    ./dev cargo run -- value --waves tests/fixtures/generated/m2_core.vcd --scope top --signals missing --at 5ns

The first command must fail with exit code 1 and include `cpu.valid`. The second must fail with exit code 1 and include `no dumped signal with basename 'missing'` plus the optimized/aliased/not-dumped causes.

Run the complete gate before final review and again after all fixes:

    ./dev just ci

Expected result: all mandatory checks and tests pass; optional FSDB checks either pass when the SDK is available or print the repository-standard skip message.

Commit without bypassing hooks:

    git status --short
    git diff --check
    git commit -m "fix(signal): suggest valid query paths"

After reviews and WIP cleanup, push and open the PR:

    git push -u origin dev3-84/signal-path-diagnostics
    gh pr create --repo kleverhq/wavepeek --base dev3 --head dev3-84/signal-path-diagnostics --title "fix(signal): suggest valid query paths" --body-file tmp/pr-84.md

### Validation and Acceptance

A scoped wrong path with an exact basename elsewhere recursively under the scope fails with a relative, copyable candidate. An unscoped wrong path fails with canonical candidates. A close basename typo receives ranked candidates. Suggestions never exceed five and are stable across repeated runs. An absent basename receives no suggestions and explicitly mentions optimized, aliased, or undumped RTL. Ambiguous FSDB paths and entries unusable by the failed lookup surface are not suggested.

`value`, `change`, `property`, generic extraction, and protocol extraction mappings all route through the shared direct or expression diagnostic. Expression failures preserve `fatal: expr`, semantic code, source excerpt, and span while including the shared suggestion or absence text. Direct failures preserve `fatal: signal`. Every failure exits 1 and leaves stdout empty, including JSON and JSONL invocations.

VCD and FST commands over equivalent generated fixtures produce the same diagnostic. Optional FSDB tests demonstrate the same policy when Verdi is available. `./dev just ci` passes.

### Idempotence and Recovery

All test, formatting, and documentation commands are safe to rerun. Generated fixture files remain under ignored `tests/fixtures/generated/`; disposable logs and PR text remain under ignored `tmp/`. Do not delete unrelated files in `tmp/`. If a commit hook fails, keep the container running, fix the reported problem, rerun the narrow check, and retry the commit without `--no-verify`. Review agents are read-only; all fixes occur in this worktree.

### Artifacts and Notes

The target human diagnostics are:

    fatal: signal: signal 'valid' not found under scope 'top'
    closest query names:
      cpu.valid

and:

    fatal: signal: signal 'missing' not found under scope 'top'
    no dumped signal with basename 'missing'; the RTL declaration may be optimized, aliased, or not dumped

Expression diagnostics retain their existing envelope and insert the same message beneath `semantic:EXPR-SEMANTIC-UNKNOWN-SIGNAL` rather than changing category.

### Interfaces and Dependencies

No dependency will be added. `src/waveform/mod.rs` will own candidate ranking and rendering because it already provides backend-neutral hierarchy and resolution access. The direct contextual resolver will accept parallel canonical/query slices and optional scope, validate their lengths, preserve request order, and return `Vec<ResolvedSignal>`. The expression contextual resolver will accept canonical path, displayed query name, and optional scope and return `ExprResolvedSignal`.

`WaveformExprHost` will carry only the fixed optional scope for one command execution; this is not configurable policy or a new abstraction. Existing raw `resolve_signals` and `resolve_expr_signal` methods remain the low-level backend operations used for candidate validation and internal callers that do not represent user query boundaries.

Revision note (2026-08-14 05:02Z): Recorded completed implementation, tests, documentation, full quality evidence, the discovered protocol-local naming constraint, and its candidate-depth decision before the first implementation commit.
