# Remove Codex Web environment support

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

WavePeek currently maintains a separate Codex Web bootstrap that duplicates the supported container environment. After this change, maintainers have one development path: the local development container and its CI profile. A maintainer can verify the result by confirming that `tools/codex/` is absent, `just --list` has no Codex setup or resume recipes, current repository guidance has no Codex Web setup instructions, and the standard container quality gates pass.

## Non-Goals

This work does not remove the Codex command-line tool from the development container, its `/home/ubuntu/.codex` persistent state mount, host-side state preparation or tests, the `openai-codex` provider under `tools/agent/`, either devcontainer profile, or published changelog history. The repository-external Codex Web configuration is also excluded because the maintainer will update it separately.

## Progress

- [x] (2026-08-12 05:27Z) Read issue #93, repository guidance, current automation, environment documentation, breadcrumb policy, and quality-gate instructions.
- [x] (2026-08-12 05:27Z) Located current Codex Web setup references and separated them from Codex CLI state support that must remain.
- [ ] Delete the Codex Web helpers and remove their `justfile` variables and recipes.
- [ ] Update current maintainer documentation, comments, breadcrumbs, and the Unreleased changelog while preserving published history and Codex CLI state support.
- [ ] Validate focused acceptance checks, `just dev-setup`, `just check`, and `just ci` in the development container.
- [ ] Commit implementation milestones and remove this branch-local plan before handoff.
- [ ] Run parallel correctness/docs and KISS+YAGNI+ponytail reviews, address findings, and run a fresh control review.
- [ ] Push the branch and open a pull request targeting `dev3`.

## Surprises & Discoveries

- Observation: The issue's external Codex Web setup command is not stored in the repository.
  Evidence: The maintainer explicitly said they will update it and instructed this work not to access it.

- Observation: Current Codex references include both removable Codex Web bootstrap support and retained Codex CLI state support.
  Evidence: `tools/codex/`, `just codex-setup`, and `just codex-resume` are removable, while `.devcontainer/devcontainer.json` mounts `/home/ubuntu/.codex` and `tools/repo/test_devcontainer_initialize.py` verifies host-side state preparation.

## Decision Log

- Decision: Make deletion and direct text edits only; add no compatibility recipes, replacement bootstrap, tests, dependencies, or abstractions.
  Rationale: Issue #93 explicitly requests removal without compatibility or replacement tooling, and existing quality gates can verify this documentation-and-automation deletion.
  Date/Author: 2026-08-12 / Pi

- Decision: Preserve all Codex CLI state references and tests that describe the supported devcontainer behavior.
  Rationale: The issue explicitly distinguishes Codex CLI support from the Codex Web bootstrap being removed.
  Date/Author: 2026-08-12 / Pi

- Decision: Do not modify published changelog sections even though they name removed commands.
  Rationale: The issue and changelog policy require published history to remain immutable; only `Unreleased` receives a removal entry.
  Date/Author: 2026-08-12 / Pi

- Decision: Leave repository-external Codex Web configuration to the maintainer.
  Rationale: The maintainer explicitly owns that acceptance step and instructed this work not to access it.
  Date/Author: 2026-08-12 / Pi

## Outcomes & Retrospective

Work is in progress. The intended outcome is complete removal of repository-owned Codex Web bootstrap support with the existing devcontainer, CI profile, and Codex CLI persistence unchanged.

## Context and Orientation

The repository root `justfile` is the supported command entrypoint. It currently defines script-path variables and public recipes for Codex Web setup and resume; these entries must be deleted. `tools/codex/` contains the duplicated shell bootstrap and its documentation; the entire directory must be deleted.

Current maintainer guidance lives in `docs/dev/environment.md` and `docs/dev/automation.md`. Root `AGENTS.md` and `.devcontainer/AGENTS.md` are path-scoped breadcrumbs for coding agents. They currently mention the removed bootstrap or coupling and must be made accurate without removing retained Codex CLI state guidance. `.devcontainer/env_contract.sh` and `.devcontainer/Dockerfile` have comments coupling container versions to the Codex Web bootstrap; only those comments need changing because the values remain the container contract.

`CHANGELOG.md` follows Keep a Changelog. A concise `Removed` entry belongs under `## [Unreleased]`; older version sections must remain byte-for-byte unchanged. `.devcontainer/devcontainer.json`, `.devcontainer/initialize.sh`, and tests under `tools/repo/` implement Codex CLI state persistence and must remain.

A development container is the repository-defined environment built from `.devcontainer/Dockerfile`. Its local profile is `.devcontainer/devcontainer.json`; CI and release automation use `.devcontainer/devcontainer.ci.json`. `just dev-setup` verifies local tools and installs hooks. `just check` is the local handoff gate, while `just ci` includes tests and coverage.

## Open Questions

There are no repository-scoped open questions. The maintainer will remove the external Codex Web setup command separately.

## Plan of Work

First, delete `tools/codex/` and remove `codex_setup_script`, `codex_resume_script`, `codex-setup`, and `codex-resume` from `justfile`. Do not replace them.

Second, delete the Codex Cloud Setup section from `docs/dev/environment.md`, remove the recipes from `docs/dev/automation.md`, and rewrite only current comments or breadcrumbs that claim coupling to Codex Web. Keep references to Codex CLI state mounts and host-side preparation. Add a `Removed` subsection and issue-linked entry under `CHANGELOG.md` Unreleased, without editing published release text.

Third, search the current tree for removable terms and inspect the retained Codex references. Confirm `tools/codex/` is absent, `just --list` omits setup/resume recipes, container profiles and workflows are unchanged, and Codex CLI package/state/provider references remain. Run `just dev-setup`, `just check`, and `just ci` in the development container.

Finally, commit the implementation, request independent parallel reviews for correctness/documentation and KISS+YAGNI+ponytail simplicity, fix substantive findings, rerun affected checks, and request a fresh control review. Update this plan throughout, then remove it in a final cleanup commit because `docs/tracker/wip/` is branch-local. Push the branch and create a GitHub pull request against `dev3`.

### Concrete Steps

Run all commands from the repository root `/home/esynr3z/projects/wavepeek/.worktrees/wavepeek/dev3-93-remove-codex-web`.

Remove the obsolete helper directory and edit the named files. Then run:

    test ! -e tools/codex
    WAVEPEEK_IN_CONTAINER=1 just --list | grep -Ei 'codex-(setup|resume)' && exit 1 || true
    rg -n -i 'codex cloud|codex web|codex setup|codex resume|codex-setup|codex-resume|tools/codex|codex helper' --hidden --glob '!.git/**' --glob '!target/**' .

The first command must succeed. The second must emit no matching recipe. The search may show immutable published changelog history only; current docs, comments, breadcrumbs, and automation must have no match.

Verify retained support with:

    rg -n '@openai/codex|/home/ubuntu/[.]codex|wavepeek-dev/codex|openai-codex' .devcontainer tools
    git diff -- .devcontainer/devcontainer.json .devcontainer/devcontainer.ci.json .devcontainer/initialize.sh tools/repo

The first command must show the package, state mount/preparation, and provider. The diff command must show no unintended changes to retained state behavior or container profiles.

Run the repository gates inside the development container:

    just dev-setup
    just check
    just ci

All commands must exit zero. Optional FSDB checks may report a documented skip when Verdi is unavailable.

### Validation and Acceptance

Acceptance is demonstrated when `tools/codex/` does not exist; `just --list` has no Codex setup or resume recipes; current docs, comments, and breadcrumbs contain no Codex Web setup instruction or coupling; and the Unreleased changelog records the removal while published history remains untouched. The retained-search output must still identify `@openai/codex`, `/home/ubuntu/.codex`, host-side Codex state handling, and `tools/agent/`'s `openai-codex` provider.

`just dev-setup` must complete in the local development container. Existing host-side state tests exercised through `just ci` must pass, proving the persistence preparation remains operational. `just check` and `just ci` must both exit zero. Inspection of `.github/workflows/` must confirm CI and release workflows still select `.devcontainer/devcontainer.ci.json`.

The external Codex Web command is the only acceptance item not demonstrated in this repository; the maintainer owns it by explicit agreement.

### Idempotence and Recovery

Deletion and text edits are safe to reapply after inspecting `git status`. The quality commands may be rerun without changing tracked source, apart from ignored generated files under `tmp/` and ignored fixture outputs. If a gate fails, keep its output under a uniquely named repository `tmp/` path, fix only the cause, and rerun the focused command before the complete gate. Git commits provide rollback points; do not use destructive resets or delete unrelated files in `tmp/`.

### Artifacts and Notes

The initial search identified removable references in `tools/codex/`, `justfile`, `docs/dev/environment.md`, `docs/dev/automation.md`, root `AGENTS.md`, `.devcontainer/AGENTS.md`, `.devcontainer/env_contract.sh`, and one `.devcontainer/Dockerfile` comment. It also identified published history in `CHANGELOG.md`, which must remain.

### Interfaces and Dependencies

No new interface or dependency is introduced. The public `just` interface loses only `codex-setup` and `codex-resume`. The remaining development interface is `just dev-setup` inside `.devcontainer/devcontainer.json`; CI and release retain `.devcontainer/devcontainer.ci.json`. Codex CLI remains installed as `@openai/codex`, persists state at `/home/ubuntu/.codex`, and remains available as the `openai-codex` provider under `tools/agent/`.

Revision note (2026-08-12 05:27Z): Created the initial self-contained execution plan after repository and issue inspection; recorded the maintainer-owned external configuration as an explicit non-goal.
