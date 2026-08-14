# Add scope-relative signal paths to machine output

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

After this change, callers of `wavepeek signal --scope <scope> --json` or `--jsonl` can consume a signal path relative to the exact selected scope without reconstructing it from the canonical path. For example, querying `--scope top` returns `relative_path: "clk"` for `top.clk` and `relative_path: "cpu.valid"` for `top.cpu.valid`. The existing leaf `name`, canonical `path`, and human-readable output remain unchanged.

## Non-Goals

This work does not restore the JSON Schema subsystem removed in v3, change waveform backend path construction, change signal filtering, change human output, or add a new abstraction or dependency. “Output schema” in issue #82 means the runtime serialization data transfer object, its contract tests, and the normative machine-output documentation.

## Progress

- [x] (2026-08-14 04:44Z) Read issue #82, repository guidance, signal execution path, current JSON/JSONL contract, tests, and packaged documentation.
- [x] (2026-08-14 04:44Z) Confirm with the maintainer that removed JSON Schema generation must not be restored.
- [x] (2026-08-14 04:53Z) Added one shared scope-relative value to signal engine entries and exposed it through the existing JSON/JSONL contract data transfer object.
- [x] (2026-08-14 04:53Z) Updated focused VCD, FST, JSONL, and FSDB contract tests plus required in-source constructors.
- [x] (2026-08-14 04:53Z) Updated packaged signal and machine-output documentation and the Unreleased changelog.
- [ ] Run focused tests and `./dev just ci`, then commit the implementation (completed: focused VCD/FST, JSONL, FSDB, and full CI all pass; remaining: commit).
- [ ] Run first review wave with separate Luna Max code/architecture and tests/docs lanes, fix findings, verify, and commit.
- [ ] Run second review wave with separate Terra High lanes over the same areas, fix findings, verify, and commit.
- [ ] Run independent Sol High control review, fix any substantive findings, and run final `./dev just check` and `./dev just ci`.
- [ ] Remove this branch-local plan, commit final cleanup, push the branch, and open a pull request for issue #82.

## Surprises & Discoveries

- Observation: JSONL already serializes the same `crate::contract::output::SignalEntry` used by JSON.
  Evidence: `src/contract/stream.rs` converts engine signal entries with `SignalEntry::try_from`, so no separate calculation is needed.

- Observation: JSON Schema support was intentionally deleted after issue #82 was written.
  Evidence: closed issue #89 and the current tree contain runtime contract DTOs but no schema generation path.

- Observation: The local environment includes a usable Verdi FSDB Reader SDK, so optional backend parity was fully exercised rather than skipped.
  Evidence: `./dev just test-fsdb` passed 20 FSDB CLI tests, and `./dev just ci` reported `ok: fsdb: Verdi FSDB Reader SDK found`.

## Decision Log

- Decision: Compute `relative_path` once in `src/engine/signal.rs` from the exact CLI `scope` and carry it on the engine signal entry.
  Rationale: All VCD, FST, and FSDB listings converge there, while waveform backends should remain responsible only for canonical hierarchy data. One calculation also guarantees JSON/JSONL parity.
  Date/Author: 2026-08-14 / Pi

- Decision: Keep the renderer-only `display` field separate from the new machine contract field.
  Rationale: Human display depends on `--recursive`, while `relative_path` is required for every machine entry, including immediate non-recursive children.
  Date/Author: 2026-08-14 / Pi

- Decision: Interpret issue #82's “output schema” as runtime DTO, tests, and normative documentation only.
  Rationale: The maintainer confirmed that JSON Schema generation removed by issue #89 must not return.
  Date/Author: 2026-08-14 / Pi

## Outcomes & Retrospective

Implementation and review are pending. At completion this section will record the observed CLI behavior, gate results, review outcomes, commit state, and pull request URL.

## Context and Orientation

WavePeek is a Rust command-line program that reads VCD, FST, and optionally FSDB waveform files. A canonical path is the complete hierarchy path stored in a waveform, such as `top.cpu.valid`. A relative path is the portion below the exact scope supplied to `--scope`, such as `cpu.valid` when the selected scope is `top`.

`src/cli/signal.rs` parses signal command arguments. `src/engine/signal.rs::run` opens a backend-neutral `Waveform`, lists direct or recursive signals, filters and limits them, and creates engine `SignalEntry` values. The engine currently computes a renderer-only `display` by stripping `<scope>.` only for recursive human output. `src/contract/output.rs::SignalEntry` converts engine entries into JSON rows. `src/contract/stream.rs` reuses that same contract type for JSONL data records. `src/output.rs` owns human and stream rendering.

`tests/signal_cli.rs` covers VCD and FST JSON behavior, including recursive hierarchy. `tests/jsonl_cli.rs` covers representative signal JSONL rows. `tests/fsdb_cli.rs` covers the optional FSDB backend when the SDK is available. `skills/wavepeek/references/signal.md` is the command guide and `skills/wavepeek/references/machine-output.md` is the normative machine contract. `CHANGELOG.md` records user-visible Unreleased behavior.

All Cargo, formatting, test, and quality commands run through the repository-root `./dev` wrapper. The standard behavior-change gate is `./dev just ci`; the final local handoff gate is `./dev just check`. Host Git commits require the worktree container to be running and hooks installed with `./dev --install-hooks`.

## Open Questions

There are no unresolved product questions. JSON Schema generation remains out of scope by maintainer confirmation.

## Plan of Work

First, extend `src/engine/signal.rs::SignalEntry` with an owned `relative_path: String`. In `run`, derive it for every listed signal by stripping the already-built exact `scope_prefix` from its canonical path and falling back to the signal basename only as a defensive invariant-preserving behavior. Reuse this calculation for recursive human display where practical, but do not make human output depend on serialization and do not change non-recursive display behavior.

Second, extend `src/contract/output.rs::SignalEntry` with a borrowed `relative_path` field and fill it from the engine entry. Because `src/contract/stream.rs` already uses this type, JSON and JSONL will serialize the same value without another path helper. Update every direct engine `SignalEntry` constructor in tests to compile and update the JSONL contract assertion.

Third, update integration tests. VCD and FST exact JSON arrays must include immediate-child relative paths. A recursive JSON assertion must prove that a descendant retains its child scope component. The representative JSONL assertion must prove the same field appears. Optional FSDB assertions must verify the field through the same engine path without adding backend-specific logic. Keep coverage focused and reuse existing fixtures.

Fourth, update `skills/wavepeek/references/signal.md` so its JSON example and non-obvious behavior describe both canonical and relative paths. Update `skills/wavepeek/references/machine-output.md` with the signal row shape and field meanings. Add one concise entry to the current Unreleased section in `CHANGELOG.md`.

Fifth, run focused tests, formatting, and the full CI recipe. Commit a conventional `feat(signal): ...` change that closes issue #82. Then conduct two focused review waves. Each wave has a code/architecture lane and a tests/docs lane; every reviewer must inspect for correctness in its lane and also apply KISS, YAGNI, and ponytail-review principles. Wave one uses Luna Max, wave two uses Terra High over the same lanes. Apply substantive findings in the main session, rerun affected checks, update this plan, and commit fixes. A fresh Sol High reviewer then performs a full control pass. Finish with both standard gates, remove this WIP plan, commit cleanup if necessary, push, and create the pull request.

### Concrete Steps

Run all commands from `/home/esynr3z/projects/wavepeek/.worktrees/wavepeek/82`.

Before the first commit, ensure hooks and the worktree container are ready:

    ./dev --install-hooks

After editing source and focused tests, format and run focused suites:

    ./dev cargo fmt --all
    ./dev cargo test --test signal_cli
    ./dev cargo test --test jsonl_cli

If the FSDB SDK is available, run:

    ./dev just test-fsdb

Run the behavior-change gate before the implementation commit:

    ./dev just ci

Expect all mandatory checks to pass. Optional FSDB output may explicitly say it was skipped when no usable SDK is installed.

Inspect and commit the implementation without bypassing hooks:

    git diff --check
    git status --short
    git commit -am "feat(signal): add relative machine paths"

Reviewers inspect the committed branch diff against `origin/dev3`. After each fix, rerun the narrow affected tests and `./dev just ci` before committing. After the Sol High control review, run:

    ./dev just check
    ./dev just ci

Remove only this plan, preserving `docs/wip/AGENTS.md`, commit the cleanup, push the current branch, and open the pull request:

    rm docs/wip/issue-82-relative-signal-path.md
    git push -u origin HEAD
    gh pr create --base dev3 --fill

### Validation and Acceptance

Run `wavepeek signal` through the integration tests and observe that each JSON and JSONL signal row has `relative_path`. For `--scope top`, `top.clk` must serialize with `name: "clk"`, `path: "top.clk"`, and `relative_path: "clk"`. A recursive `top.cpu.valid` row must serialize with `name: "valid"`, `path: "top.cpu.valid"`, and `relative_path: "cpu.valid"`. Existing human output must remain `clk` for direct rows and `cpu.valid` for recursive descendants unless `--abs` requests canonical paths.

The exact JSON tests must pass for both generated VCD and FST fixtures. JSONL must contain the same row payload as JSON. When the optional FSDB environment is present, its focused tests must prove the same contract. `./dev just ci` and the final `./dev just check` must pass. Review is complete only when both focused waves and the independent control reviewer report no unresolved substantive findings.

### Idempotence and Recovery

Formatting, fixture preparation, tests, and quality gates are safe to rerun. The implementation changes only derived output data and documentation; it does not modify waveform files or persistent external state. If a test fails, retain its output under repository-root `tmp/` with a unique issue-82 filename, fix the smallest root cause, and rerun the narrow test before the full gate. Do not reset or delete unrelated user work. If push or PR creation fails, keep the local commits intact and retry the failed host command.

### Artifacts and Notes

The required row examples are:

    {"name":"clk","path":"top.clk","relative_path":"clk","kind":"wire","width":1}

    {"name":"valid","path":"top.cpu.valid","relative_path":"cpu.valid","kind":"wire","width":1}

The branch starts at `d3409f0` on `dev3-82/relative-signal-path`, based on `origin/dev3`.

### Interfaces and Dependencies

No dependency changes are allowed or needed. At completion, `crate::engine::signal::SignalEntry` must expose `pub relative_path: String`, and `crate::contract::output::SignalEntry` must serialize a borrowed `relative_path: &str`. `crate::contract::stream::StreamDataRow` must continue using the output contract DTO unchanged so JSON and JSONL cannot diverge. The existing standard-library `str::strip_prefix` operation is the only path transformation required.

Revision note (2026-08-14): Initial plan created after repository exploration and maintainer confirmation that removed JSON Schema support stays out of scope.

Revision note (2026-08-14 04:53Z): Recorded implementation, documentation, focused test, FSDB, and full CI completion before the implementation commit.
