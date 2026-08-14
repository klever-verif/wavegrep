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
- [x] (2026-08-14 05:24Z) Run parallel Luna Max reviews for correctness/tests, documentation/contracts, architecture/KISS, and performance; fix the substantive candidate-validity, bounded-work, documentation, JSONL, and repeated protocol adapter findings; focused VCD/FST/FSDB tests pass.
- [x] (2026-08-14 05:31Z) Run parallel Terra High reviews over the same four areas; fix unscoped protocol candidate filtering, protocol documentation/error wording, and avoidable edit-distance allocation; focused protocol tests pass.
- [x] (2026-08-14 05:49Z) Run independent Sol High control review and targeted confirmations; fix payload resolver ordering plus backend-specific direct event validity, with focused Wellen payload/event and FSDB event tests passing.
- [x] (2026-08-14 05:51Z) Run the final `./dev just ci` quality gate; all formatting, lint, build, unit, CLI, docs, coverage, and FSDB checks passed with 93.22% average source coverage and 22 FSDB CLI tests.
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

- Observation: successful path resolution alone does not make an FSDB candidate usable.
  Evidence: Luna Max correctness and architecture reviewers found that `resolve_signal` accepts real/string and unsupported FSDB encodings before sampling rejects them. Candidate validation now resolves the expression type, applies the existing backend support check, and requires a direct integral-like type for direct surfaces; `fsdb_value_rejects_non_bit_vector_signal` proves `tem` does not suggest unusable `temp`.

- Observation: sorting every matching hierarchy entry made the output bound independent from the work bound.
  Evidence: Luna Max performance review identified O(K log K) memory/time for K common-basename matches. Candidate selection now keeps only the best five while scanning and uses thresholded edit distance with length and row-minimum exits.

- Observation: protocol candidate locality applies only when a scope is selected.
  Evidence: Terra High correctness review found that the local-only resolver also removed every dotted canonical candidate from unscoped mappings. The filter now rejects dots only with `--scope`, and the APB test proves an unscoped typo suggests `top.uart_apb_p_addr_o`.

- Observation: protocol docs and errors previously used “scope-relative” for a stricter direct-member rule.
  Evidence: Terra High documentation review identified ambiguous wording in `extract.md` and all five protocol engines. They now distinguish direct local explicit mappings from scope-relative include candidates.

- Observation: generic payload diagnostic order determines which candidate capability is advertised.
  Evidence: Sol High control review found expression resolution ran before direct payload resolution, allowing a Wellen real signal typo to receive a suggestion that direct payload output rejects. Direct resolution now runs first, and `extract_generic_does_not_suggest_unusable_payload_signals` covers this boundary.

- Observation: direct event value support differs by backend encoding.
  Evidence: Sol High confirmation found that an integral-only whitelist wrongly removed usable Wellen events, while removing it alone admitted unsupported FSDB events. Candidate validation now combines shared expression support, direct resolution, and an FSDB direct-encoding check; focused Wellen `tick` and converted FSDB event tests cover both outcomes.

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

- Decision: Limit scoped protocol mapping candidates to direct members of the selected scope and reject dotted displayed leaf names.
  Rationale: Protocol mapping validation currently rejects any dotted input, including an escaped local name containing a literal dot. Suggesting recursive descendants or dotted leaves would violate the issue requirement that every suggestion be valid in the active naming mode; generic and expression lookups remain recursive.
  Date/Author: 2026-08-14, coding agent, refined after Luna Max review.

- Decision: Keep one single-path contextual protocol resolver instead of repeating one-element bulk adapters in five engines.
  Rationale: Luna Max architecture review identified the repeated slices plus `remove(0)` as avoidable glue. The shared method preserves one policy and deletes command-local adapters.
  Date/Author: 2026-08-14, coding agent.

- Decision: Validate direct candidates through existing backend capabilities rather than a shared type whitelist.
  Rationale: Wellen accepts raw events as direct values, while FSDB can expose event metadata with an unsupported direct encoding. Resolution plus backend support checks describe actual usability without duplicating type policy.
  Date/Author: 2026-08-14, coding agent after Sol High control review.

## Outcomes & Retrospective

The feature is implemented and verified. One waveform-facade policy emits at most five deterministic, copyable candidates or a concrete absent-from-dump explanation across direct, expression, payload, protocol, VCD/FST, and FSDB lookups. Backend ambiguity and unsupported encodings remain their original errors, expression diagnostics retain their envelope, and machine failures retain exit code 1 with empty stdout before streaming begins.

Luna Max reviewed correctness/tests, docs/contracts, architecture/KISS, and performance in parallel; Terra High repeated the same four lanes after fixes; Sol High performed the independent control review and targeted confirmations. Their substantive findings were fixed and covered by focused tests. Final `./dev just ci` passed: 666 library tests, all CLI suites, 22 FSDB CLI tests, docs checks, and 93.22% average source coverage. Only branch-local plan cleanup, push, and PR publication remain.

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

Revision note (2026-08-14 05:51Z): Recorded the passing final quality gate and completed outcome after all review fixes; only required WIP cleanup and PR publication remain.
