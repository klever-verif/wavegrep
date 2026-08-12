# Collapse development into one tool-only container

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

WavePeek contributors and automation currently use two container profiles, while the local profile also provisions GUI software, coding agents, nested container tooling, persistent agent state, and local GitHub credentials. After this change, local commands, CI, release checks, and docs staging use the same credentialless tool image. From any directory in a checkout or linked Git worktree, a contributor can run commands through `./dev`, for example `./dev just check`; the wrapper starts only that worktree's container, preserves the caller's directory and process behavior, and mounts Verdi only when a valid host `VERDI_HOME` is supplied.

## Non-Goals

This work does not move Git commits or hooks to the host, implement the Codex Web bootstrap tracked by issue #93, add replacement container profiles, preserve container GUI support, or add another environment layer around `./dev`.

## Progress

- [x] (2026-08-12 05:58Z) Read issue #96, repository guidance, container definitions, workflow entrypoints, quality recipes, and existing helper-test patterns.
- [x] (2026-08-12 06:35Z) Collapsed `.devcontainer/Dockerfile` and `.devcontainer/devcontainer.json` into one tool-only image and removed obsolete lifecycle, agent-state, GUI, and GitHub-auth files.
- [x] (2026-08-12 06:35Z) Added the root `./dev` wrapper and eight focused tests for checkout discovery, linked worktrees, current-directory preservation, optional mounts, stale-mount refusal, argument and process I/O forwarding, signals, exit status, and `--exec-only`.
- [x] (2026-08-12 06:35Z) Pointed CI, release, and docs workflows at the sole configuration and switched the docs push to job-scoped checkout authentication.
- [x] (2026-08-12 06:35Z) Updated maintainer documentation and breadcrumbs for the unified container and host entrypoint.
- [x] (2026-08-12 10:34Z) Built the image and ran `./dev just dev-setup`, eight focused wrapper tests, converter checks, clean-image exclusions, `./dev just ci`, and FSDB checks with the mounted Verdi SDK. The no-Verdi path is covered by focused mount tests and the FSDB helper suite.
- [x] (2026-08-12 10:35Z) Committed the implementation and Verdi runtime correction through the normal pre-commit and commit-msg hooks.
- [ ] Run parallel correctness/docs and KISS+YAGNI+ponytail reviews, fix findings, rerun affected checks, and complete an independent control review. (Completed: three first-pass reviews and corrections for docs authentication, host/container runbook boundaries, stale profile wording, and ponytail simplifications; remaining: independent control review.)
- [ ] Remove this branch-local plan, push the branch, and open a pull request targeting `dev3`.

## Surprises & Discoveries

- Observation: The installed Devcontainer CLI's built-in `--mount-git-worktree-common-dir` supports only worktrees created with relative paths, while this checkout's `.git` file points to an absolute Git directory.
  Evidence: `devcontainer up --help` states the relative-path restriction; `git rev-parse --path-format=absolute --git-common-dir` returns `/home/esynr3z/projects/wavepeek/.git` for this linked worktree.
- Observation: `devcontainer exec` has no working-directory option.
  Evidence: `devcontainer exec --help` lists workspace and container selection but no directory flag, so the wrapper must use a minimal in-container `cd` followed by `exec`.
- Observation: The docs push currently depends on the repository credential helper being removed.
  Evidence: `.github/workflows/docs.yml` disabled checkout credentials, passed tokens into the container, and ran `.devcontainer/setup-github-auth.sh` before `push-staged`.
- Observation: Removing all former development packages broke the command-line Verdi VCD-to-FSDB converter because it dynamically loads legacy X11 libraries despite being used without a GUI.
  Evidence: The first FSDB fixture gate failed with `libXt.so.6: cannot open shared object file`; restoring `libxt6t64`, `libxmu6`, and `libnuma1` made `./dev just ci` pass all FSDB tests.

## Decision Log

- Decision: Use one final Docker stage with no named `dev` or `ci` target, and let `.devcontainer/devcontainer.json` build the Dockerfile's final stage.
  Rationale: This is the smallest configuration that proves there is one image target and prevents another profile split.
  Date/Author: 2026-08-12 / Pi
- Decision: Keep the wrapper as one root Bash script and test it with Python's standard `unittest` plus fake `devcontainer` and `docker` executables.
  Rationale: Bash preserves command arguments and process streams directly; the repository already discovers `tools/repo/test_*.py`, so no dependency or test framework is needed.
  Date/Author: 2026-08-12 / Pi
- Decision: Identify runtime containers with the Devcontainer CLI's native absolute-workspace labels and inspect Docker bind mounts before reuse.
  Rationale: Native labels already provide one container per absolute worktree. Docker inspection is necessary to reject stale Git-common-directory or Verdi mounts without deleting user state.
  Date/Author: 2026-08-12 / Pi
- Decision: In the docs push job, rely on the ephemeral credentials configured by `actions/checkout` instead of passing tokens into the container or installing a helper.
  Rationale: The checkout credential is job-scoped and already attached to the checked-out repository; it avoids rebuilding local credential plumbing.
  Date/Author: 2026-08-12 / Pi

## Outcomes & Retrospective

The source, wrapper, workflows, and maintainer documentation now implement one credentialless command-line container. The rebuilt image passes the repository gate, including FSDB compilation, conversion, and CLI tests with a valid mounted Verdi SDK. Three focused reviews completed; their substantive findings were corrected and affected tests pass. The independent control pass remains.

## Context and Orientation

`.devcontainer/Dockerfile` currently builds `surfer_builder`, `rtl_artifacts`, `node_toolchain`, `base`, `ci`, and `dev` stages. The `dev` stage adds GitHub CLI, Verdi command wrappers, hooks, GUI libraries, Surfer, Node.js, Codex, Claude Code, Pi, and a nested Devcontainer CLI. `.devcontainer/devcontainer.json` selects `dev` and mounts local agent, Verdi, and GitHub state; `.devcontainer/devcontainer.ci.json` selects `ci`. Host initialization lives in `.devcontainer/initialize.sh`, container credential setup in `.devcontainer/setup-github-auth.sh`, and local token setup in `tools/repo/setup_github_env.sh`.

The root `justfile` is the stable task interface. Its `require-container` recipe requires `WAVEPEEK_IN_CONTAINER=1`; `dev-setup`, `check`, and `ci` are the principal setup and quality commands. `just test-aux` discovers Python unit tests under `tools/repo/`. GitHub Actions workflows under `.github/workflows/` currently create a temporary alias for `.devcontainer/devcontainer.ci.json` because the nonstandard filename cannot be passed directly to older Devcontainer tooling.

A linked Git worktree is a checkout whose administrative Git directory lives in another checkout's common `.git` directory. Its checked-out `.git` file can contain an absolute path. To make Git work inside a container, `./dev` must bind-mount that common directory at the identical absolute path in addition to mounting the worktree at `/workspaces/<worktree-basename>`.

## Open Questions

There are no user-blocking questions. Package removal and mount inspection details will be validated against real image and container behavior before finalizing them.

## Plan of Work

First, simplify `.devcontainer/Dockerfile` to retain Ubuntu, Rust and Cargo tools, C/C++ compilation, Python and docs tooling, actionlint, hooks, GitHub CLI, Icarus Verilog, GTKWave converters, benchmark tooling, RTL fixtures, and command-line Verdi wrappers in one final stage. Remove Surfer, Node.js, all coding agents, the nested Devcontainer CLI, GUI libraries, interactive packages unused by repository commands, and obsolete version pins. Simplify `.devcontainer/devcontainer.json` to the workspace bind, host networking, container marker, user mapping, and setup command; delete `.devcontainer/devcontainer.ci.json`, lifecycle/auth scripts, their tests, and local auth documentation.

Second, add executable root file `dev`. It will resolve the enclosing worktree with Git, derive the caller's relative path, validate `VERDI_HOME` when set, and add explicit bind mounts for a linked worktree's Git common directory and optional Verdi. Before normal startup it will query Docker for the container selected by Devcontainer's absolute-workspace labels and compare the relevant bind mounts. A mismatch will fail with an explicit `devcontainer up --remove-existing-container ...` command. Normal mode runs `devcontainer up` and then `devcontainer exec`; `--exec-only` skips every startup and inspection path that could alter a container and invokes only `devcontainer exec`. The in-container command will `cd` to `/workspaces/<basename>/<caller-relative-path>` and then `exec` the original argument vector.

Third, add `tools/repo/test_dev.py`. Tests will create temporary main and linked Git worktrees and fake external executables that record exact calls. They will prove selection from nested directories, distinct absolute workspace identities, common-directory and Verdi mounts, invalid Verdi rejection, stale mount refusal and recreation text, argument boundaries, stdin/stdout, signals, exit status, and the absence of startup operations in `--exec-only` mode.

Fourth, update all workflows to use `.devcontainer/devcontainer.json` directly and remove alias setup. The docs push job will retain job-scoped `actions/checkout` authentication and stop passing GitHub tokens or invoking the deleted helper in the container. Update `AGENTS.md`, `.devcontainer/AGENTS.md`, `docs/dev/environment.md`, `docs/dev/automation.md`, `docs/dev/quality.md`, `docs/dev/fsdb.md`, `docs/dev/git.md`, and related wording to describe one tool-only container and `./dev`.

Finally, rebuild through `./dev`, run all acceptance checks, commit validated slices, request focused parallel reviews including a challenge review dedicated to KISS, YAGNI, and ponytail deletion opportunities, fix substantive findings, run a fresh control review, remove this WIP plan, push, and open a PR against `dev3` referencing issue #96.

### Concrete Steps

Run all commands from the repository root unless a test explicitly changes directory:

    chmod +x dev
    python3 -B -m unittest tools/repo/test_dev.py
    ./dev just dev-setup
    ./dev sh -c 'vcd2fst --help >/dev/null && fst2vcd --help >/dev/null'
    ./dev just check
    ./dev just ci

When host `VERDI_HOME` is unset, `./dev just check-fsdb-env` must print a `skip: fsdb:` message and succeed. When it names a valid SDK, `/opt/verdi/share/FsdbReader` must be visible and the FSDB portions of `just check` and `just ci` must pass.

Inspect the clean runtime after build:

    ./dev sh -c 'for tool in node npm devcontainer codex claude pi surfer; do ! command -v "$tool"; done'
    docker inspect <container-id>

The inspection must show no agent-state or GitHub-credential mounts, and no `/opt/verdi` mount when host `VERDI_HOME` is unset.

### Validation and Acceptance

The repository must contain only `.devcontainer/devcontainer.json` as a devcontainer configuration and the Dockerfile must have one final tool image. All four workflows must reference that file directly. `./dev` must work from the checkout root, a nested directory, and a linked worktree. Tests must demonstrate exact argument boundaries, stdin and output forwarding, signal delivery, and exit propagation. `--exec-only` tests must show no `devcontainer up`, Docker removal, build, start, restart, or recreation call.

A rebuilt image must provide `vcd2fst` and `fst2vcd`, while `command -v` must fail for Node.js, the nested Devcontainer CLI, Codex, Claude Code, Pi, and Surfer. Docker inspection must show only required workspace, linked-worktree Git common-directory, and optional Verdi mounts, with no local GitHub or agent state. `./dev just dev-setup`, `./dev just check`, and `./dev just ci` must exit zero. FSDB checks must skip cleanly without Verdi and pass if a valid SDK is available.

### Idempotence and Recovery

File edits and tests are repeatable. `./dev` may reuse a correctly mounted container but never deletes or recreates one automatically. If mounts are stale, use the exact recreation command printed by the wrapper, then rerun the original command. Disposable logs belong under `tmp/` and existing unrelated files there must not be removed. Git commits must use normal hooks; failed hooks are fixed and retried rather than bypassed.

### Artifacts and Notes

Expected stale-mount behavior is an error naming the changed mount and a directly runnable command shaped like:

    devcontainer up --workspace-folder '<absolute-worktree>' --remove-existing-container ...

Expected no-Verdi behavior is:

    skip: fsdb: VERDI_HOME is not set

Focused host validation currently passes:

    python3 -B -m unittest tools/repo/test_dev.py
    Ran 8 tests in 0.862s
    OK

Image and gate evidence:

    ./dev just dev-setup       # passed during container post-start
    ./dev just ci              # passed, including 20 FSDB CLI tests
    vcd2fst/fst2vcd checks     # passed
    forbidden-tool checks      # node/npm/devcontainer/codex/claude/pi/surfer absent

Review evidence will be added as implementation proceeds.

### Interfaces and Dependencies

The new public developer interface is:

    ./dev [--exec-only] COMMAND [ARG ...]

`COMMAND` is mandatory. No other wrapper options or configuration files are introduced. The implementation uses Bash, Git, Docker CLI, and the already host-installed Devcontainer CLI. Tests use only Python's standard library. The container keeps existing pinned versions in `.devcontainer/env_contract.sh` only for tools it still installs.

Plan revision note (2026-08-12): Initial self-contained plan created after repository and issue research; implementation and validation evidence remain to be recorded.

Plan revision note (2026-08-12 06:35Z): Recorded the completed implementation milestones and focused wrapper-test evidence before image validation.

Plan revision note (2026-08-12 10:36Z): Recorded successful image, converter, clean-tool, full CI, FSDB, hook, and commit evidence plus the Verdi runtime-library discovery.

Plan revision note (2026-08-12 10:58Z): Recorded first-pass correctness, docs, and mandatory KISS+YAGNI+ponytail review progress and the resulting authentication, runbook, wording, Dockerfile, wrapper, and test simplifications.
