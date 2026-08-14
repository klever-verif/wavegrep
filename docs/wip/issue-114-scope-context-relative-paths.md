# Expose scoped machine-output context and relative paths

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

WavePeek machine output currently loses the selected `--scope` after signal resolution. As a result, a consumer cannot interpret an empty scoped result and must manually remove scope prefixes from canonical signal paths. After this change, scoped `signal`, `value`, `change`, and `extract generic` commands will expose the exact selected scope in JSON `context.scope` and JSONL `begin.context.scope`. Scoped value-bearing rows will retain canonical `path` and add `relative_path`, while unscoped output and human-readable output remain unchanged.

A user can observe the behavior with the generated `m2_core.vcd` fixture: `value --scope top --signals clk --json` will report `context: {"scope":"top"}` and a signal containing both `path: "top.clk"` and `relative_path: "clk"`. JSONL will carry the same context in its first record.

## Non-Goals

This work does not add CLI flags, scope aliases, backend normalization, new dependencies, or relative paths to protocol extractor mappings and payloads. It does not change signal resolution, human-readable rendering, diagnostics, ordering, or unscoped machine schemas. It does not redesign the protocol extractor context objects.

## Progress

- [x] (2026-08-14 14:15Z) Read issue #114, repository guidance, output contracts, command engines, streaming sinks, and existing tests.
- [x] (2026-08-14 14:15Z) Choose the smallest design: one optional scope on command results, one boundary-safe relative-path helper, and direct JSONL scope begin support.
- [x] (2026-08-14 14:31Z) Implement scoped context and relative paths for `signal`, `value`, `change`, and `extract generic` in JSON and JSONL.
- [x] (2026-08-14 14:31Z) Add focused contract and CLI coverage, update packaged machine-output documentation, and pass 129 focused CLI tests plus the JSON/JSONL parity test.
- [x] (2026-08-14 14:36Z) Commit the implementation as `43a4023` after `./dev just ci` passed, including 22 FSDB integration tests and all pre-commit hooks.
- [x] (2026-08-14 14:53Z) Run four parallel Luna Max lanes with KISS/YAGNI and ponytail-review; correctness was clean, docs/parity/allowlist/human-allocation findings were fixed, focused CLI tests and 665 library tests passed, and all three impacted lanes rechecked clean.
- [x] (2026-08-14 14:59Z) Run four parallel Terra High lanes with the same KISS/YAGNI and ponytail-review principles; three lanes were clean, unscoped generic parity was added, focused tests passed, and the impacted contract lane rechecked clean.
- [x] (2026-08-14 15:05Z) Run an independent Sol High control review; clarify one low-severity generic-extract sentence, receive a clean recheck, and pass final `./dev just ci`, including enabled FSDB coverage.
- [ ] Remove this branch-local plan, commit cleanup, push the branch, and open a PR against `dev3` that closes issue #114.

## Surprises & Discoveries

- Observation: `signal` already computes the required relative paths but discards its selected scope before output serialization.
  Evidence: `src/engine/signal.rs::run` fills `SignalEntry::relative_path`, while `src/contract/output.rs::OutputContextData::from_command_data` returns no context for `signal`.

- Observation: JSONL `change` and `extract generic` start streaming inside command-specific sinks before a `CommandResult` exists.
  Evidence: `JsonlChangeSink::start` and `JsonlExtractSink::start` call `JsonlWriter::begin()`, so scope must be supplied at sink start rather than adapted after collection.

- Observation: current scope lookup is exact and accepts canonical dot-separated paths; there is no alias normalization layer to preserve.
  Evidence: Wellen uses `lookup_scope` on components and FSDB indexes scopes by canonical string.

- Observation: protocol extractors reuse the generic extraction sink trait, so changing that trait's `start` signature would spread scope handling into unrelated protocol modules.
  Evidence: the first compile check rejected four protocol sink implementations after a draft signature change; keeping scope on `JsonlExtractSink` preserved the existing shared trait and removed that coupling.

- Observation: the full CI gate includes enabled FSDB integration tests whose exact scoped JSON expectations must evolve with the public contract.
  Evidence: the first full gate exposed five stale FSDB expectations; after adding scope-relative fields, `./dev just test-fsdb` passed all 22 tests and the repeated full `./dev just ci` passed.

- Observation: relative paths are a machine-only contract for `value`, `change`, and `extract generic`; human rendering uses display labels and `--abs` instead.
  Evidence: Luna performance review found avoidable per-row allocations in human mode. Gating relative-path construction on JSON/JSONL preserves output and avoids those allocations.

## Decision Log

- Decision: Treat a successfully validated `--scope` token as the canonical selected scope.
  Rationale: Both backends perform exact canonical lookup. A new resolver API would add unused flexibility without changing behavior.
  Date/Author: 2026-08-14, coding agent.

- Decision: Store `scope: Option<String>` directly on `CommandResult` and add a small scope context contract variant.
  Rationale: Output context is command-wide metadata, and this avoids wrapping each command's existing data in new one-use container types.
  Date/Author: 2026-08-14, coding agent.

- Decision: Compute `relative_path` from resolved canonical paths with one boundary-safe helper that removes only the exact `scope.` prefix.
  Rationale: User query tokens can be relative or canonical and can repeat; resolved paths are the stable source of truth. Boundary-safe stripping prevents `top` from matching `topology`.
  Date/Author: 2026-08-14, coding agent.

- Decision: Omit both scope context and row `relative_path` when no `--scope` was supplied.
  Rationale: Issue #114 explicitly preserves unscoped behavior and avoids a breaking schema expansion where no relative namespace exists.
  Date/Author: 2026-08-14, coding agent.

- Decision: Keep the scope-capable command allowlist in `CommandName::supports_scope_context` and reuse it in both JSON and JSONL contracts.
  Rationale: Luna architecture review found duplicated match lists that could drift; one small predicate is the shared source without adding a layer.
  Date/Author: 2026-08-14, coding agent.

## Outcomes & Retrospective

The implementation produces scoped JSON and JSONL context and relative paths while preserving unscoped, human, and protocol behavior. Focused suites, expanded JSON/JSONL parity, 665 library tests, enabled FSDB integration, hooks, benchmark smoke, and final full CI pass. Four Luna Max lanes and four Terra High lanes reviewed correctness, contracts, architecture, and performance with KISS/YAGNI and ponytail-review; impacted lanes rechecked clean. Sol High independently controlled the consolidated branch and rechecked its one wording fix clean. Only WIP cleanup, push, and PR creation remain.

## Context and Orientation

WavePeek is a Rust command-line waveform inspector. A canonical signal path is the complete dot-separated hierarchy name, for example `top.cpu.valid`. A relative path removes the exact selected scope and separator, so the same signal under `--scope top` is `cpu.valid` and under `--scope top.cpu` is `valid`.

`src/engine/mod.rs` defines `CommandResult`, the internal result passed to output rendering. `src/contract/output.rs` converts collected command data into the strict JSON envelope. `src/contract/stream.rs` defines JSONL begin and data records. `src/output.rs` selects human, JSON, or JSONL output and owns `JsonlWriter`.

The command implementations are `src/engine/signal.rs`, `value.rs`, `change.rs`, and `extract.rs`. `signal` lists hierarchy entries and already emits required relative paths. `value` emits `ValueSignalValue`; `change` emits `ChangeSignalValue`; generic extraction emits `ExtractPayloadValue`. The latter two support incremental JSONL through sink traits, so their begin records are written during execution. Integration tests invoke the built CLI from `tests/signal_cli.rs`, `value_cli.rs`, `change_cli.rs`, `extract_generic_cli.rs`, `jsonl_cli.rs`, and `json_jsonl_parity.rs`. The normative user-facing schema is `skills/wavepeek/references/machine-output.md`.

## Open Questions

No product or implementation questions remain. Optional FSDB validation will run through repository gates when the local Verdi SDK is available; otherwise the standard gate records its skip.

## Plan of Work

First, add a private boundary-safe helper in `src/engine/mod.rs` that returns an optional relative path only when a scope exists and the canonical path begins with the exact `scope.` prefix. Add optional scope storage to `CommandResult`, filling it only for the four commands named by issue #114. Extend value, change, and generic extraction emitted values with optional relative paths computed from resolved canonical paths. Keep `signal.relative_path` required.

Second, add a scope context DTO to `src/contract/output.rs`. JSON envelopes will select this context for scoped `signal`, `value`, `change`, and `extract generic`; protocol extractor context remains unchanged. Their value-bearing row DTOs will serialize optional relative paths only when present. Extend `src/contract/stream.rs` and `JsonlWriter` with direct scope-aware begin support. Collected JSONL adaptation will read `CommandResult.scope`; streaming change and generic extraction sinks will receive the scope at start.

Third, update focused integration and parity tests. Verify immediate and nested relative paths, context on empty scoped output, JSONL begin context, JSON/JSONL parity, and omission for unscoped commands. Update `skills/wavepeek/references/machine-output.md` with the exact schema and examples. Avoid unrelated documentation or architecture changes because module responsibilities remain intact.

Commit the implementation after focused tests pass, then run `./dev just ci`. Run four read-only focused reviewers in parallel for each requested wave: correctness/tests, documentation/contracts, architecture/KISS/YAGNI, and performance. Every prompt also requires a ponytail-review pass that seeks deletions and simpler native or existing alternatives. Apply fixes only in the main worktree and commit them. Run one fresh Sol High control review over the consolidated branch, resolve substantive findings, rerun `./dev just ci`, delete this WIP plan, commit cleanup, push, and open a PR against `dev3` with `Closes #114`.

### Concrete Steps

All commands run from `/home/esynr3z/projects/wavepeek/.worktrees/wavepeek/dev3-114`.

Prepare hooks and iterate with formatting and focused tests:

    ./dev --install-hooks
    ./dev just format
    ./dev cargo test --test signal_cli
    ./dev cargo test --test value_cli
    ./dev cargo test --test change_cli
    ./dev cargo test --test extract_generic_cli
    ./dev cargo test --test jsonl_cli
    ./dev cargo test --test json_jsonl_parity

Exercise a visible JSON result:

    ./dev cargo run -- value --waves tests/fixtures/generated/m2_core.vcd --scope top --signals clk --at 5ns --json

The output must contain:

    "context":{"scope":"top"}
    "signals":[{"path":"top.clk","relative_path":"clk","value":"1'h1"}]

Run the complete behavior gate before reviews and after all review fixes:

    ./dev just ci

Expected result: mandatory checks and tests pass. Optional FSDB gates either pass when the SDK is available or print the repository-standard skip message.

Commit without bypassing hooks:

    git status --short
    git diff --check
    git commit -m "feat(output): expose scoped relative paths"

After reviews and WIP cleanup, push and open the PR:

    git push -u origin dev3-114/scope-context-relative-paths
    gh pr create --repo kleverhq/wavepeek --base dev3 --head dev3-114/scope-context-relative-paths --title "feat(output): expose scoped relative paths" --body-file tmp/pr-114.md

### Validation and Acceptance

Scoped `signal`, `value`, `change`, and `extract generic` JSON results contain `context.scope` even when `data` is empty. Their JSONL streams contain identical scope context only in the first `begin` record. Every scoped emitted signal or generic payload retains canonical `path` and includes `relative_path` computed from the exact selected scope. Immediate children use a basename and descendants retain their child scope components.

Without `--scope`, `value`, `change`, and `extract generic` omit both envelope context and row relative paths. Human output is byte-for-byte unchanged by this schema work, including `--abs`. JSON and JSONL retain matching contexts, data rows, diagnostics, and ordering. Existing protocol extractor contexts are unchanged. `./dev just ci` passes.

### Idempotence and Recovery

Formatting, tests, docs checks, and quality gates are safe to rerun. Generated waveform fixtures remain ignored under `tests/fixtures/generated/`; disposable logs and PR text belong under ignored `tmp/`. Do not delete unrelated files there. If a hook fails, keep the worktree container running, fix the reported cause, rerun the narrow check, and retry without `--no-verify`. Review agents are read-only; all edits occur in this worktree.

### Artifacts and Notes

The target scoped JSON shape is:

    {"type":"result","command":"value","context":{"scope":"top"},"data":[{"time":"5ns","signals":[{"path":"top.clk","relative_path":"clk","value":"1'h1"}]}],"diagnostics":[]}

The target scoped JSONL prefix is:

    {"type":"begin","seq":0,"command":"value","context":{"scope":"top"}}

The equivalent unscoped rows retain their current canonical `path` and omit `relative_path`.

### Interfaces and Dependencies

No dependency will be added. `src/engine/mod.rs` will expose a crate-private helper equivalent to:

    fn relative_signal_path(path: &str, scope: Option<&str>) -> Option<String>

`CommandResult` will carry `scope: Option<String>`. `ValueSignalValue`, `ChangeSignalValue`, and `ExtractPayloadValue` will carry `relative_path: Option<String>` with engine serde omission and matching contract DTO omission. `OutputContextData` will add a scope variant containing one canonical `scope` string. `JsonlWriter` will add one scope-aware begin method; existing protocol `begin_context` remains unchanged.

Revision note (2026-08-14 14:15Z): Initial plan created after issue and repository investigation; no implementation existed yet.

Revision note (2026-08-14 14:31Z): Recorded implementation, focused-test evidence, manual output evidence, and the decision to avoid changing the shared protocol sink trait.

Revision note (2026-08-14 14:34Z): Recorded the FSDB expectation discovery and successful full CI evidence.

Revision note (2026-08-14 14:52Z): Recorded Luna Max findings, fixes, focused validation, machine-only allocation decision, and shared allowlist decision.

Revision note (2026-08-14 14:53Z): Recorded clean Luna impacted-lane rechecks.

Revision note (2026-08-14 14:59Z): Recorded Terra High results, unscoped generic parity fix, focused validation, and clean impacted-lane recheck.

Revision note (2026-08-14 15:04Z): Recorded Sol High control finding and clarified that unscoped generic payloads retain canonical `path`.

Revision note (2026-08-14 15:05Z): Recorded clean Sol recheck, successful final CI, and completed outcome evidence before WIP cleanup.
