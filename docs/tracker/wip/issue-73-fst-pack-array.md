# Fix FST PACK and ARRAY attribute panic

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

`wavepeek info` currently panics when it opens a valid FST waveform containing the hierarchy `PACK` and `ARRAY` attributes emitted by newer Verilator versions. After this change, users can inspect those waveforms directly and receive the usual metadata instead of process exit code 101. The behavior is observable by running `wavepeek info --waves tests/fixtures/hand/verilator_pack_array.fst` and seeing a successful three-line metadata response ending at `60ps`.

## Non-Goals

This work does not migrate `wavepeek` to a new `wellen` API series, change any public command or schema contract, implement an FST parser locally, or broaden support for unrelated waveform constructs. It also does not add the regression fixture to the performance catalog because the dependency-only parser correction is covered functionally by a focused CLI integration test and the existing full performance gate.

## Progress

- [x] (2026-07-25 19:54Z) Read issue 73, repository testing and benchmarking guidance, and the current dependency and fixture policies.
- [x] (2026-07-25 19:54Z) Reproduced the panic on current `main` commit `caea6c39b27a1ec763b699266ef2d241f8a84c1a` with the issue fixture: exit code 101 from `fst-reader` 0.14.3 at its unimplemented PACK attribute branch.
- [x] (2026-07-25 20:00Z) Added the checked-in format-specific FST fixture, documented it in the waveform fixture policy, and added the focused `info` CLI regression test.
- [x] (2026-07-25 20:00Z) Ran the focused test before changing dependencies; it failed with process exit 101 at the `fst-reader` PACK `todo!`, as required for TDD evidence.
- [x] (2026-07-25 20:02Z) Updated the lockfile to `wellen` 0.20.4 and its required transitive versions; the focused regression, full `info_cli` suite, fixture-policy test, and direct CLI acceptance invocation all pass.
- [ ] Run the repository quality gates, obtain focused code review, address all actionable findings, and rerun affected validation.
- [ ] Fetch the latest `origin/main`, reconcile the branch if needed, and run the clean-worktree performance gate against that exact main commit.
- [ ] Finalize and remove this branch-local plan, create conventional commits, push the branch, and open a GitHub pull request that links issue 73 and reports validation and performance evidence.

## Surprises & Discoveries

- Observation: The upstream reproducer is only 1,637 bytes, so a self-contained checked-in regression fixture is smaller and more deterministic than adding network access or a fixture generator to tests.
  Evidence: Its SHA-256 is `f5f0f17576bbaf27846a911dd38f1580333b14aa7cb31fbe3d5d9accc4fd8f55` and `stat` reports 1,637 bytes.
- Observation: The manifest constraint `wellen = "~0.20"` already admits the upstream fix, while the original lockfile selected `wellen` 0.20.2 and `fst-reader` 0.14.3.
  Evidence: `cargo update -p wellen --precise 0.20.4` updated `fst-reader` to 0.16.6 and the required compression, memory-map, integer-enum, and deflate transitive packages without changing `Cargo.toml` or Rust source.

## Decision Log

- Decision: Keep the existing `wellen = "~0.20"` manifest requirement and update the lockfile to exactly `wellen` 0.20.4.
  Rationale: The existing semver requirement already expresses the supported API line. Pinning a new manifest requirement or migrating to 0.25 would add unrelated API and schema work without improving this fix.
  Date/Author: 2026-07-25 / Pi
- Decision: Check the Verilator-generated FST into `tests/fixtures/hand/` and add one human-output assertion to `tests/info_cli.rs`.
  Rationale: The attributes are binary FST metadata that the repository's ordinary Icarus-Verilog-to-FST generator does not create. The fixture policy explicitly reserves hand dumps for format-specific contracts, and invoking the built CLI proves the user-visible regression end to end.
  Date/Author: 2026-07-25 / Pi
- Decision: Preserve the exact upstream file and document its provenance and digest instead of reducing or regenerating it.
  Rationale: Exact bytes reproduce the reported issue, are already very small, and avoid introducing a Verilator fixture toolchain solely for this regression.
  Date/Author: 2026-07-25 / Pi

## Outcomes & Retrospective

The focused implementation milestone is complete: the exact issue fixture is tracked and policy-documented, its CLI test failed against the original lockfile, and `wellen` 0.20.4 now opens it with exact expected metadata and empty stderr. Full quality validation, independent review, performance comparison, and pull request publication remain.

## Context and Orientation

`wavepeek` is a Rust command-line program. `Cargo.toml` declares the `wellen` library as the common VCD/FST waveform reader, and `Cargo.lock` selects the exact library versions used by builds. On current main, `wellen` 0.20.2 selects `fst-reader` 0.14.3. That reader reaches an unimplemented branch and panics when it parses the valid FST `PACK` and `ARRAY` hierarchy attributes.

The `info` command's end-to-end integration tests are in `tests/info_cli.rs`. They use `tests/common/mod.rs::wavepeek_cmd` to execute the compiled `wavepeek` binary and `fixture_path` to find waveform files under `tests/fixtures/generated/` or `tests/fixtures/hand/`. A hand dump is a checked-in waveform whose format-specific bytes are the behavior under test. Every checked-in VCD or FST hand dump must be declared with a durable reason in `tests/fixtures/waveform_policy.json`; `tests/waveform_fixture_policy.rs` enforces that rule.

The exact regression input comes from `ekiwi/fst-reader` commit `92b3b821e88b8058a72a7cd04607a327b8397152`, path `fsts/verilator/verilator-pull-7255-t_trace_complex_structs_cc_fst.fst`. It has SHA-256 `f5f0f17576bbaf27846a911dd38f1580333b14aa7cb31fbe3d5d9accc4fd8f55`. With fixed dependencies, `info` must print `time_unit: 1ps`, `time_start: 0ps`, and `time_end: 60ps` with empty stderr.

Repository commands are run from `/workspaces/wavepeek/.worktrees/fix-issue73-wellen-update` inside the devcontainer. `just ci` is the standard full quality gate. `just bench-gate <baseline-ref> <revised-ref> auto` builds clean checkouts and compares functional output and timing; its ignored evidence is stored under `tmp/bench-gate/`. The branch-local plan itself belongs under `docs/tracker/wip/` while work is active and must be removed before merge.

## Open Questions

There are no unresolved design questions. If `origin/main` advances before the final performance run, rebase the branch onto it, rerun affected validation, and use the fetched main commit as the benchmark baseline.

## Plan of Work

### Milestone 1: Capture the regression with a failing CLI test

Copy the exact 1,637-byte upstream FST from the temporary download into `tests/fixtures/hand/verilator_pack_array.fst`. Add that path to `hand_dumps` in `tests/fixtures/waveform_policy.json` with a reason stating that its Verilator-specific PACK and ARRAY hierarchy metadata cannot be generated by the ordinary fixture pipeline. In `tests/info_cli.rs`, add a test named `info_opens_fst_with_verilator_pack_array_attributes` that invokes `wavepeek info` on the fixture, requires success and empty stderr, and compares stdout exactly to the three expected metadata lines. Run only that test while the old lockfile is still present. Acceptance for this milestone is the test failing with process exit 101 and the `not yet implemented: PACK attributes` panic, which proves it detects the reported bug rather than passing accidentally.

### Milestone 2: Apply the minimal dependency fix

Run `cargo update -p wellen --precise 0.20.4`. Inspect `Cargo.lock` and confirm that only the necessary transitive package selections changed; do not edit `Cargo.toml` because its current `~0.20` constraint already permits the fixed patch. Rerun the focused test and the fixture-policy test. Acceptance is exact successful metadata output, no stderr, and a valid manifested hand fixture. Commit the focused implementation once those tests pass.

### Milestone 3: Validate quality and review maintainability

Run `just ci` in the devcontainer and expect all static checks, auxiliary tests, Rust coverage tests, schema checks, docs checks, and available FSDB gates to pass. Request independent focused review of the complete diff for correctness, test quality, dependency scope, fixture policy, and unnecessary complexity. Resolve every actionable finding with the smallest local change and rerun the narrow affected tests plus `just ci` when changes affect code or test behavior. Acceptance is a clean review with no unresolved correctness or maintainability findings and a clean repository quality gate.

### Milestone 4: Compare performance with current main and publish the PR

Fetch `origin/main` immediately before benchmarking. If it differs from the branch base, rebase, resolve only necessary conflicts, and rerun `just ci`. With all intended source changes committed and the worktree clean, run `just bench-gate <exact-origin-main-sha> HEAD auto`. Acceptance is no functional mismatch, missing or invalid artifact, timeout, or unconfirmed timing regression in the generated summary. Record the exact refs and summary path in this plan, then update the outcome, commit that state if useful for review history, remove this WIP plan as repository guidance requires, and commit the cleanup. Run `just check` after the final history is in place, push `fix/issue73-wellen-update`, and create a PR targeting `main`. The PR must use a conventional title, include `Fixes #73`, summarize the fixture, test, and lockfile update, and report `just ci`, review, and benchmark evidence.

### Concrete Steps

From `/workspaces/wavepeek/.worktrees/fix-issue73-wellen-update`, execute:

    cp tmp/issue73-verilator-complex-structs.fst tests/fixtures/hand/verilator_pack_array.fst
    sha256sum tests/fixtures/hand/verilator_pack_array.fst
    cargo test --test info_cli info_opens_fst_with_verilator_pack_array_attributes -- --exact

Before the dependency update, the expected focused-test evidence is:

    test info_opens_fst_with_verilator_pack_array_attributes ... FAILED
    Unexpected return code, failed
    code=101
    not yet implemented: PACK attributes

Then execute:

    cargo update -p wellen --precise 0.20.4
    cargo test --test info_cli info_opens_fst_with_verilator_pack_array_attributes -- --exact
    cargo test --test waveform_fixture_policy waveform_fixture_policy_manifest_matches_repository_layout -- --exact
    just ci

The focused fixed output must report one passed test and no failures. After review fixes, fetch and benchmark against current main:

    git fetch origin main
    baseline=$(git rev-parse origin/main)
    just bench-gate "$baseline" HEAD auto

Finally, with a clean reviewed branch:

    just check
    git push -u origin fix/issue73-wellen-update
    gh pr create --repo kleverhq/wavepeek --base main --head fix/issue73-wellen-update

### Validation and Acceptance

The new test `info_opens_fst_with_verilator_pack_array_attributes` must fail on the original lockfile with exit 101 and pass after the lockfile update. A direct invocation of the fixed binary must exit zero, write nothing to stderr, and print exactly:

    time_unit: 1ps
    time_start: 0ps
    time_end: 60ps

`tests/waveform_fixture_policy.rs` must prove the binary fixture is tracked and justified. `just ci` and the post-history `just check` must pass in the devcontainer. Independent review must leave no unresolved findings. The performance gate must compare the fetched `origin/main` commit to the branch HEAD in the same controlled environment and pass its functional and timing thresholds. The final GitHub PR must be open against `main`, link issue 73 for automatic closure, and expose the commits and validation evidence to maintainers.

### Idempotence and Recovery

Copying the fixture and running Cargo or Just commands is safe to repeat. `cargo update --precise` converges on the same lockfile. Benchmark output uses timestamped ignored directories and does not overwrite source files. Do not delete unrelated files under `tmp/`. If the network download must be repeated, download to `tmp/issue73-verilator-complex-structs.fst` and verify the stated digest before copying it. If main advances, fetch and rebase before rerunning validation rather than benchmarking stale refs. If review requires edits after the performance run, commit them and rerun the performance gate because HEAD has changed.

### Artifacts and Notes

Baseline reproduction on `caea6c39b27a1ec763b699266ef2d241f8a84c1a`:

    $ cargo run --quiet -- info --waves tmp/issue73-verilator-complex-structs.fst
    thread 'main' panicked at .../fst-reader-0.14.3/src/io.rs:923:40:
    not yet implemented: PACK attributes
    [exit 101]

Focused regression test before the dependency change:

    running 1 test
    test info_opens_fst_with_verilator_pack_array_attributes ... FAILED
    Unexpected failure.
    code=101
    not yet implemented: PACK attributes

Focused and adjacent tests after the dependency change:

    test info_opens_fst_with_verilator_pack_array_attributes ... ok
    test result: ok. 1 passed; 0 failed
    test waveform_fixture_policy_manifest_matches_repository_layout ... ok
    test result: ok. 8 passed; 0 failed  # full info_cli suite

Direct fixed behavior:

    time_unit: 1ps
    time_start: 0ps
    time_end: 60ps
    stderr bytes: 0

Fixture integrity:

    f5f0f17576bbaf27846a911dd38f1580333b14aa7cb31fbe3d5d9accc4fd8f55  tests/fixtures/hand/verilator_pack_array.fst
    1637 bytes

### Interfaces and Dependencies

No Rust interface changes are required. `tests/common/mod.rs::fixture_path(&str) -> PathBuf` remains the fixture resolver, and the existing `wavepeek info --waves <path>` public CLI remains the tested interface. `Cargo.toml` remains unchanged with `wellen = "~0.20"`. `Cargo.lock` must select `wellen` 0.20.4 and the compatible `fst-reader` version that contains PACK and ARRAY parsing support.

Revision note (2026-07-25): Created the plan after repository research and baseline reproduction so implementation, TDD evidence, review, performance comparison, and PR publication can continue from this file alone. Updated it after adding the fixture and test to preserve the observed red-test evidence before changing dependencies, then after the minimal lockfile update to record focused acceptance results and exact transitive dependency effects.
