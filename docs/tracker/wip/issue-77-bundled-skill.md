# Replace embedded docs with one bundled skill

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with the repository's `exec-plan` skill.

## Purpose / Big Picture

WavePeek currently ships two overlapping offline documentation surfaces: `wavepeek docs` and `wavepeek skill`. After this change, the installed binary exposes only `wavepeek skill <DIRECTORY>`, which creates a complete, version-matched skill package on disk. A user can point the command at a missing or empty directory and receive `SKILL.md`, the existing user documentation under `references/`, an empty `examples/` directory, and a provenance manifest. The same extracted references feed the documentation website, so the repository has one packaged user-documentation source.

## Non-Goals

This work does not change the crate version. It does not install the skill into any agent harness, detect harnesses, merge package updates, overwrite non-empty directories, retain compatibility aliases for `wavepeek docs`, or add example workflows before concrete examples are requested.

## Progress

- [x] (2026-08-12 18:58Z) Read issue #77, repository guidance, current docs/skill runtime, tests, and website tooling; resolve requirements with the maintainer.
- [x] (2026-08-12 18:58Z) Write this executable plan and start the worktree devcontainer with host Git hooks installed.
- [x] (2026-08-12 19:18Z) Move the canonical package to `skills/wavepeek/`, remove YAML topic front matter, and make all internal links relative.
- [x] (2026-08-12 19:18Z) Replace the docs and single-file skill runtimes with safe full-tree extraction and provenance metadata.
- [x] (2026-08-12 19:18Z) Remove every product-facing and source-code trace of the old `docs` command and update help/contracts/tests.
- [x] (2026-08-12 19:18Z) Generate the website from extracted `references/` and update README and maintainer guidance.
- [x] (2026-08-12 19:49Z) Run focused tests and `just ci`, then commit the implementation as `f8d4227`.
- [ ] Run Luna Max focused review lanes, fix findings, and commit (completed: runtime/test and package-doc lanes; fixed destination install race/recovery, invalid README command, stale help path, and added symlink coverage; one tooling reviewer was stopped after failing to return promptly; remaining: bounded replacement tooling lane and review-fix commit).
- [ ] Run Terra High focused review lanes over the same areas, fix findings, and commit.
- [ ] Run a Sol High control review, fix findings, and commit.
- [ ] Complete final `just ci`/`just check` evidence, remove this WIP plan, push the branch, and open a PR against `dev3`.

## Surprises & Discoveries

- Observation: `include_dir`, already used for `docs/public`, recursively embeds a directory and can remain the sole embedding dependency.
  Evidence: `src/docs/mod.rs` declares `include_dir!("$CARGO_MANIFEST_DIR/docs/public")`.
- Observation: the existing docs exporter has replacement behavior that is deliberately broader than issue #77 permits; reusing it wholesale would preserve unwanted complexity.
  Evidence: the removed `src/docs/mod.rs::export_catalog` accepted managed replacement with `--force`, while the new command rejects every non-empty destination.
- Observation: website navigation previously depended on deleted YAML fields, but stable directory groups plus each page's H1 are sufficient.
  Evidence: `just docs-site-build` prepared 22 references and completed strict MkDocs validation after front matter removal.
- Observation: removing the topic catalog made the production `serde_yaml` dependency unused.
  Evidence: `cargo check` succeeds after removing `serde_yaml`; Cargo.lock also drops its transitive YAML-only packages.
- Observation: an unconstrained first-wave tooling reviewer failed to terminate after extensive investigation and was explicitly stopped.
  Evidence: the Luna Max lane remained active after 2.8M tokens and 282 tool calls; a replacement lane is bounded by turns.

## Decision Log

- Decision: Keep the package version at its current value.
  Rationale: The maintainer stated the major-version bump will be handled separately.
  Date/Author: 2026-08-12, maintainer and coding agent.
- Decision: Treat the command destination itself as the skill root.
  Rationale: `wavepeek skill /tmp/pkg` should create `/tmp/pkg/SKILL.md`, not `/tmp/pkg/wavepeek/SKILL.md`.
  Date/Author: 2026-08-12, maintainer and coding agent.
- Decision: Permit a missing directory or a directory with no entries; reject any directory containing visible or hidden entries.
  Rationale: This is the simplest literal interpretation of “new or empty” and guarantees no merge or overwrite.
  Date/Author: 2026-08-12, maintainer and coding agent.
- Decision: Move the current `docs/public` tree unchanged in topical layout beneath `skills/wavepeek/references`, except remove YAML front matter and delete the obsolete docs-command topic.
  Rationale: The maintainer requested preservation of the tree without metadata that existed only for the catalog runtime.
  Date/Author: 2026-08-12, maintainer and coding agent.
- Decision: Create an empty `examples/` directory and no `scripts/` or `assets/` directories.
  Rationale: No concrete examples, scripts, or assets are currently required; empty directories need an explicit tracked placeholder.
  Date/Author: 2026-08-12, maintainer and coding agent.
- Decision: Remove all traces of the product command named `docs`, while retaining ordinary repository concepts such as maintainer docs, documentation-site tooling, and `docs.rs` package metadata.
  Rationale: The maintainer explicitly prohibited traces of the removed command, not the ordinary word “docs” in unrelated contexts.
  Date/Author: 2026-08-12, maintainer and coding agent.
- Decision: Attempt atomic rename over an existing empty destination before using the remove-and-rename fallback required by platforms that cannot replace an empty directory.
  Rationale: This closes the ordinary concurrent-creator race on platforms that support directory replacement while retaining cross-platform acceptance of existing empty directories and preserving any concurrently created content.
  Date/Author: 2026-08-12, coding agent after Luna Max runtime review.
- Decision: Do not add symlink policing to the internal MkDocs staging helper.
  Rationale: It consumes a package just produced by the trusted current binary; additional validation would not protect a user trust boundary and is outside issue #77.
  Date/Author: 2026-08-12, coding agent after Luna Max runtime review.

## Outcomes & Retrospective

Implementation has not started. At completion this section will compare extracted package behavior, old-command removal, website generation, test evidence, review results, and PR state against the purpose above.

## Context and Orientation

The repository is a Rust CLI. `src/cli/mod.rs` defines top-level Clap commands and help. `src/engine/mod.rs` dispatches parsed commands. `src/cli/docs.rs`, `src/engine/docs.rs`, and most of `src/docs/mod.rs` implement the old topic catalog, search, display, and export system. `src/cli/skill.rs` currently has no arguments, and `src/engine/skill.rs` prints one Markdown file from `docs/skills/wavepeek.md`.

The existing user topics live under `docs/public/`. Each starts with YAML front matter used by the old runtime catalog. The compact router lives separately at `docs/skills/wavepeek.md`. Integration tests are in `tests/docs_cli.rs`, `tests/skill_cli.rs`, and `tests/cli_contract.rs`; unit-only docs assertions also appear under `src/tests/`, `src/output.rs`, and `src/contract/output.rs`.

The documentation website is built by `just docs-site-build`. It currently runs `wavepeek docs export`, then `tools/docs/prepare_mkdocs.py` validates the old manifest and stages topics for MkDocs. The website publication flow under `tools/docs/` and `.github/workflows/docs.yml` consumes that staging interface.

A “bundle format version” is an integer in the extracted `manifest.json` that identifies the manifest/tree contract independently from the WavePeek semantic version. A “non-empty directory” is one for which `std::fs::read_dir` yields at least one entry, including hidden files.

## Open Questions

None. The maintainer resolved versioning, destination layout, empty examples, front matter removal, and complete removal of the old command.

## Plan of Work

First create `skills/wavepeek/`, move the short router to `SKILL.md`, and move every still-relevant file from `docs/public/` beneath `references/`. Strip every YAML header because catalog IDs and sections no longer drive runtime behavior. Rewrite the router and reference cross-links to use only relative paths. Remove the old docs-command reference because that interface no longer exists. Track the intentionally empty `examples/` directory with a neutral `.gitkeep` file; the materializer must include it.

Next replace `src/docs/mod.rs` with the smallest bundle module, renamed to `src/skill.rs` so no removed-command subsystem remains. Embed `skills/wavepeek` recursively with `include_dir`. Define one serializable manifest containing the bundle format version and `env!("CARGO_PKG_VERSION")`. Implement extraction by validating that the destination is absent or an empty directory, staging the complete output in a unique sibling directory, writing all embedded files plus `manifest.json`, creating embedded empty directories, and atomically renaming the staged tree into place. If an existing empty destination must be accepted, remove only that verified-empty directory immediately before rename. On failure, clean the stage and preserve or recreate the empty destination where practical. Do not add overwrite flags or update machinery.

Then add the required `PathBuf` positional argument in `src/cli/skill.rs`, make `src/engine/skill.rs` call extraction, and remove `src/cli/docs.rs`, `src/engine/docs.rs`, all docs command variants, output DTOs, renderers, tests, fixtures, dependencies, and help references. Exact syntax/default guidance belongs in Clap help, while narrative help points to relative bundle references after extraction rather than a runtime browsing command.

Update `tools/docs/prepare_mkdocs.py` to consume a materialized skill root and copy `references/` into generated MkDocs input. Keep only validation needed by the current package: manifest kind/version, WavePeek version, required `SKILL.md`, `references/`, and `examples/`, safe paths, and website navigation. Because YAML headers are gone, define website navigation in the staging tool from the stable existing path layout and Markdown H1 headings rather than introducing a second metadata format. Change `just docs-site-build` to run `cargo run -- skill <temporary bundle root>`. Update helper tests and publication wording without changing release topology.

Finally update README, architecture/style/testing/automation/quality guidance, and breadcrumbs to name `skills/wavepeek` as the one user documentation source. Run a repository-wide search proving the removed command has no code, tests, generated help, or user docs left. Commit coherent milestones. Conduct three required review stages: parallel Luna Max code, docs/tooling, and architecture/simplicity lanes; parallel Terra High lanes over the same areas after Luna fixes; then one fresh Sol High control pass. Every reviewer is read-only and must apply KISS, YAGNI, and ponytail-review principles in addition to its lane focus. Apply substantive findings in the main session and rerun affected checks.

### Concrete Steps

All commands run from the repository root `/home/esynr3z/projects/wavepeek/.worktrees/wavepeek/dev3-77-bundled-skill`.

Prepare the environment and establish baseline state:

    ./dev --install-hooks
    ./dev true
    git status --short --branch

After content/runtime edits, run narrow tests:

    ./dev cargo test --test skill_cli --test cli_contract
    ./dev cargo test --lib
    ./dev python3 -B -m unittest tools.docs.test_prepare_mkdocs

Exercise the binary directly:

    rm -rf tmp/issue-77-skill
    ./dev cargo run --quiet -- skill tmp/issue-77-skill
    find tmp/issue-77-skill -type f -o -type d | sort
    ./dev cargo run --quiet -- skill tmp/issue-77-skill

The first extraction must succeed and produce `SKILL.md`, `references/`, `examples/`, and `manifest.json`. The second must fail with a deterministic `fatal: file:` message because the destination is non-empty.

Prove the old command is absent with searches targeted at the product interface:

    rg -n 'wavepeek docs|DocsArgs|CommandName::Docs|commands/docs|wavepeek-docs-export' . \
      -g '!CHANGELOG.md' -g '!docs/tracker/wip/issue-77-bundled-skill.md'
    ./dev cargo run --quiet -- docs

The search must have no matches. The command must fail as an unrecognized subcommand and point only to current help.

Run website and full gates:

    ./dev just docs-site-check
    ./dev just ci
    ./dev just check

After all reviews and fixes, remove this plan, commit the cleanup, push `dev3-77/bundled-skill`, and open a PR against `dev3` with `gh pr create` referencing `Closes #77`.

### Validation and Acceptance

`wavepeek skill <DIRECTORY>` succeeds when the directory does not exist and when it exists with zero entries. It creates the same complete embedded package in both cases. `manifest.json` identifies WavePeek's current package version and bundle format version 1. A directory containing any entry is rejected without modifying that entry. Tests compare extracted files byte-for-byte with `skills/wavepeek`, verify hidden-entry rejection, verify no partial destination after induced invalid cases where feasible, and inspect manifest fields.

The extracted package contains `SKILL.md`, every retained reference file, and an empty `examples/`. All Markdown links that point within the package are relative and resolve after extraction. No file tells an agent to invoke the removed command. `wavepeek --help` lists `skill` but not `docs`; `wavepeek docs` is a Clap argument error. No docs command implementation, catalog/search/show/export DTO, compatibility path, fixture, or dependency remains.

`just docs-site-check` proves the website source comes from a skill extracted by the current built binary. `just ci` proves Rust behavior, source coverage, tooling tests, actions, and docs publication checks. `just check` proves formatting, lint, build, docs-site checks, hooks-related checks, and optional FSDB compilation where available.

### Idempotence and Recovery

Content moves and source edits are version-controlled and can be retried. Tests use temporary directories. Manual extraction uses only `tmp/issue-77-skill`, which may be removed explicitly without touching other `tmp/` content. The extraction implementation stages into a unique sibling and rejects non-empty destinations before writing, so retrying after a user-facing validation error is safe. Never recursively delete an unverified destination.

If a gate fails, fix the narrow failure and rerun it before the full gate. If a review worker fails without usable output, restart that same lane and do not count it as completed. Host Git commits run only while the already-started worktree container is available, so hooks can execute through `./dev --exec-only` without rebuilding.

### Artifacts and Notes

Issue #77 requires one canonical package and explicitly removes the old command rather than retaining compatibility. The resolved output shape is:

    <DIRECTORY>/
    ├── SKILL.md
    ├── references/
    ├── examples/
    └── manifest.json

The intended manifest is deliberately small:

    {
      "bundle_format_version": 1,
      "wavepeek_version": "2.2.3"
    }

A stable `kind` field may be retained only if website validation needs to distinguish this manifest from unrelated JSON; no topic catalog belongs in it.

### Interfaces and Dependencies

Keep the existing `include_dir` crate because it already embeds recursive trees and represents empty directories. Keep `serde` and `serde_json` for deterministic manifest serialization. Remove `serde_yaml` if no remaining production source uses it. Do not add dependencies.

In `src/skill.rs`, expose only the runtime operation needed by the engine, with a shape equivalent to:

    pub const BUNDLE_FORMAT_VERSION: u32 = 1;
    pub fn materialize(destination: &Path) -> Result<(), WavepeekError>;

In `src/cli/skill.rs`, `SkillArgs` owns one required `directory: PathBuf`. In `src/engine/skill.rs`, `run` calls `crate::skill::materialize` and returns a short human success result. No JSON mode, force option, nested command, or installation behavior is added.

Plan revision note: Initial plan written after repository investigation and maintainer decisions; it defines the complete implementation, validation, review, commit, and PR workflow for issue #77.
