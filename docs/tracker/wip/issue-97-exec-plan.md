# Run host Git hooks through the worktree devcontainer

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

After this change, a contributor can keep Git identity, signing, credentials, commits, pushes, issues, and pull requests on the host while repository checks remain in the worktree's credentialless devcontainer. The contributor explicitly installs reviewed hook copies once with `./dev --install-hooks`, explicitly starts that worktree's container, and can then run `git commit` without Rust, Pre-commit, or Commitizen installed on the host. The hooks use only the already-running container and validate the exact Git index and commit-message file supplied by the host Git process.

## Non-Goals

This work does not add another wrapper for ordinary development commands, start or rebuild containers from hooks, forward arbitrary host Git environment variables, install host credentials or tools, or make hook installation automatic during container startup.

## Progress

- [x] (2026-08-12 13:13Z) Read issue #97, issue #96, repository guidance, the current `./dev` execution path, hook configuration, quality recipes, documentation, and helper tests.
- [x] (2026-08-12 13:13Z) Started this worktree's devcontainer explicitly and confirmed the existing generated host hooks contain `/opt/wavepeek-python/bin/python3`.
- [x] (2026-08-12 13:27Z) Added explicit idempotent host-hook installation and reviewed hook dispatch with safe path translation and environment filtering.
- [x] (2026-08-12 13:27Z) Fixed commit-message filename handling and made `./dev --exec-only` failures actionable without allowing lifecycle operations.
- [x] (2026-08-12 13:27Z) Added Docker-free focused tests and an opt-in Docker-backed main/linked-worktree smoke test.
- [x] (2026-08-12 13:27Z) Updated maintainer documentation and breadcrumbs to describe the host/container boundary and normal workflow.
- [ ] Run focused and full gates, commit logical milestones, conduct parallel correctness/docs and mandatory KISS+YAGNI+ponytail challenge reviews, fix findings, and run an independent control review. (Completed: all gates and initial parallel reviews; corrected the critical container-writable active-hook location and accepted KISS reductions; remaining: re-review, control review, and final cleanup.)
- [ ] Remove this branch-local plan, push the branch, and open a pull request targeting `dev3`.

## Surprises & Discoveries

- Observation: `just dev-setup` currently rewrites shared Git hooks every time the devcontainer starts.
  Evidence: `.devcontainer/devcontainer.json` runs `just dev-setup` as `postStartCommand`, and that recipe runs `pre-commit install`; the resulting host files contain the container-only path `/opt/wavepeek-python/bin/python3`.
- Observation: linked worktrees keep their Git directory under the common directory, while the common directory is mounted into their container at the same absolute path.
  Evidence: this worktree reports `/home/esynr3z/projects/wavepeek/.git/worktrees/dev3-97-git-hooks` as its Git directory and `/home/esynr3z/projects/wavepeek/.git` as its common directory; `dev` adds the latter as a same-path bind mount.
- Observation: the existing auxiliary test discovery can include an opt-in Docker smoke test without adding a recipe, while skipping it inside the normal container gate.
  Evidence: `./dev just test-aux` discovered 20 `tools/repo` tests and reported one Docker smoke skip.
- Observation: the Git common directory is writable from linked-worktree containers, so it cannot safely hold active host hook executables.
  Evidence: the correctness reviewer identified host-code execution from container-modified hooks; installation now targets host data storage, and both focused and Docker tests prove that path is outside container mounts.

## Decision Log

- Decision: Add `./dev --install-hooks` as the sole explicit host installation command and install copies under `${XDG_DATA_HOME:-$HOME/.local/share}/wavepeek/git-hooks/<repository-id>` with an absolute repository-local `core.hooksPath`.
  Rationale: Git and `./dev` already exist on the host; this avoids a dependency or another ordinary-command wrapper, shares activation across linked worktrees, prevents branch switches from replacing active hook code before reinstall, and keeps active host executables outside every container-writable mount.
  Date/Author: 2026-08-12 / Pi
- Decision: Use one reviewed Bash dispatcher copied as both `pre-commit` and `commit-msg`, alongside a copied `dev` dispatcher.
  Rationale: The two hooks differ only in the Pre-commit stage arguments, and Bash/Git are already required by the host workflow.
  Date/Author: 2026-08-12 / Pi
- Decision: Reconstruct only `GIT_WORK_TREE`, `GIT_DIR`, `GIT_COMMON_DIR`, and `GIT_INDEX_FILE` for the container command, and explicitly forward only `SKIP` as a hook control.
  Rationale: These paths are required to inspect the host invocation's exact repository state; unrelated host Git variables can redirect object/config/credential behavior and must not cross the boundary.
  Date/Author: 2026-08-12 / Pi

## Outcomes & Retrospective

Implementation is in progress.

## Context and Orientation

The root `dev` Bash script is the only host entrypoint for container commands. Its default mode may call `devcontainer up`; `--exec-only` selects an existing container and must never perform lifecycle operations. It maps each absolute worktree to `/workspaces/<worktree-basename>`. For a linked worktree, it additionally mounts Git's common directory—the shared `.git` storage used by all worktrees—at the same absolute path inside the container.

`.pre-commit-config.yaml` defines repository-local Pre-commit and commit-message stages. The root `justfile` owns their commands. Today `just dev-setup` calls `pre-commit install` inside the container, which creates host-visible launchers in the bind-mounted Git directory with a container-only Python path. The commit-message hook also suppresses filenames, while `just check-commit` always guesses `COMMIT_EDITMSG` instead of accepting Git's supplied file.

The reviewed hook source lives at `tools/repo/git-hook`. `./dev --install-hooks` copies that source and `dev` itself into per-repository host data storage outside tracked worktree files and container mounts, then activates the copies through `core.hooksPath`. Focused host-side tests live with existing repository helper tests under `tools/repo/`; the standard `just test-aux` discovery already includes `test_*.py` there.

## Open Questions

There are no product questions. Implementation will use the smallest path mapping that accepts descendants of the mounted worktree or Git common directory and rejects every other path.

## Plan of Work

First, extend `dev` with a host-only installation mode before its normal command and container logic. Resolve the physical common directory, derive a stable per-repository directory under host data storage, refuse an effective `core.hooksPath` that is neither empty nor the exact managed location, copy reviewed executable files idempotently, and write the absolute repository-local configuration. Remove generated-hook installation from `just dev-setup` and from container lifecycle documentation.

Next, add `tools/repo/git-hook`. It will identify its stage from its installed filename, derive and normalize the host worktree, Git directory, common directory, index, and commit-message paths, translate worktree descendants to `/workspaces/<basename>` while preserving same-path common-directory descendants, reject outside paths, clear all inherited `GIT_*` variables, and execute its sibling `dev --exec-only`. The inner command will set only reconstructed Git paths plus optional `SKIP`, then run the correct Pre-commit stage. `exec` and the existing `dev` process bridge preserve streams, signals, and exit status.

Then change `just check-commit` to accept an optional message path whose manual default remains Git's `COMMIT_EDITMSG`, and let Pre-commit pass the real commit-message filename. Improve `dev --exec-only` diagnostics for missing host tools and absent or stopped containers, always including the explicit `./dev true` startup instruction and never calling `devcontainer up` in that mode.

Add focused tests for safe installation, custom-path refusal, main and linked worktree mapping, default and temporary indexes, partial staged content, commit-message paths, environment filtering, process I/O and status, signals, and no-container/tool failures. Add an opt-in host Docker smoke test that creates an isolated checkout and linked worktree, explicitly starts both containers, installs hooks, and exercises valid and invalid host commits. It will remove only resources it created.

Finally, update `AGENTS.md`, `.devcontainer/AGENTS.md`, `docs/dev/environment.md`, `docs/dev/automation.md`, `docs/dev/quality.md`, `docs/dev/git.md`, and `tools/repo/README.md`. Run all issue-required gates, request focused read-only reviews in parallel—including a mandatory KISS+YAGNI+ponytail reviewer that challenges every added layer and line—apply justified findings, and run a fresh control review before removing this plan and opening the PR.

### Concrete Steps

From the repository root, implement and validate in this order:

    ./dev --exec-only python3 -m unittest tools/repo/test_git_hooks.py
    WAVEPEEK_RUN_DOCKER_HOOK_SMOKE=1 python3 -m unittest tools/repo/test_git_hooks_docker.py
    ./dev just test-aux
    ./dev just pre-commit
    ./dev just check
    ./dev just ci

After installation, prove normal host behavior with an already-running container:

    ./dev --install-hooks
    ./dev just check
    git commit -m "chore: update development workflow"
    git push

A stopped worktree container must instead produce a short failure containing:

    start this worktree explicitly with: ./dev true

### Validation and Acceptance

The Docker-free suite must prove installation is idempotent, does not contact Docker, and refuses an unrelated effective hooks path. Hook probes must observe exact mapped worktree, Git-directory, common-directory, normal/temporary index, and message paths; a partial-index probe must read staged content rather than the working file. Outside paths and unsupported Git environment variables must be rejected or absent. Process probes must preserve stdin, stdout, stderr, signals, and non-zero status.

The opt-in Docker smoke must make a valid conventional host commit in both a main checkout and linked worktree using their distinct already-running containers, and reject an invalid message through the real container Commitizen. No host Rust, Pre-commit, or Commitizen executable may be used.

The final repository gates `./dev just test-aux`, `./dev just pre-commit`, `./dev just check`, and `./dev just ci` must pass. Documentation must consistently assign agents/Git/credentials/remote operations to the host and repository tools/gates to each worktree container.

### Idempotence and Recovery

`./dev --install-hooks` may be rerun after any reviewed hook change. It replaces only its managed copies and accepts only its own already-configured path; it leaves an unrelated custom `core.hooksPath` untouched and fails with a remediation message. Tests use temporary repositories and clean only their own paths and containers. If hook activation must be removed manually, a maintainer can unset the repository-local path with `git config --local --unset core.hooksPath`; the installer will then work again without deleting custom files.

### Artifacts and Notes

The installed layout is:

    ${XDG_DATA_HOME:-$HOME/.local/share}/wavepeek/git-hooks/<repository-id>/dev
    ${XDG_DATA_HOME:-$HOME/.local/share}/wavepeek/git-hooks/<repository-id>/pre-commit
    ${XDG_DATA_HOME:-$HOME/.local/share}/wavepeek/git-hooks/<repository-id>/commit-msg

The source remains reviewable at root `dev` and `tools/repo/git-hook`; branch switches do not alter the active copies until `./dev --install-hooks` is rerun.

### Interfaces and Dependencies

No dependency is added. Host execution uses Bash, Git, Docker, and the Devcontainer CLI already required by `./dev`. Container execution uses the existing Pre-commit and Commitizen installations.

`dev` gains exactly one host interface:

    ./dev --install-hooks

`just check-commit` gains one optional positional argument:

    just check-commit [message-path]

The installed hook interface remains Git's standard executable `pre-commit` and `commit-msg <message-path>` contract.
