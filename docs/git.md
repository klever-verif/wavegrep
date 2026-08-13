# Git and Contribution Hygiene

Use conventional commits. Install reviewed host hooks explicitly with `./dev --install-hooks`; the commit-msg hook runs container Commitizen through `just check-commit`. Start the current worktree's container before `git commit`, because hooks use `./dev --exec-only` and never perform container lifecycle operations.

Commit small logical milestones. For broad refactors, commit after each independently validated slice so stale-reference fallout can be found and reverted without excavating a single heroic rubble pile.

Do not bypass hooks with `--no-verify` unless the user or maintainer explicitly asks. If a hook fails, fix the cause, rerun the relevant command, and retry the commit.

Use repository-root `tmp/` for ignored scratch files, logs, and ad hoc outputs. Do not globally clean it or delete arbitrary existing files because another agent or the user may own them.

Use `docs/wip/` for branch-local tracked artifacts that need review or must survive across agent sessions. Those artifacts should be removed before merging to the default branch unless a maintainer intentionally keeps them for handoff.

## GitHub and Fork Remotes

Fork contributors should keep `origin` pointed at their fork and use `upstream` for `https://github.com/kleverhq/wavepeek.git`. Agents, Git identity and signing, credentials, pushes, issues, and pull requests remain on the host; the credentialless container runs repository tools and quality gates only.

Commands that intentionally target the upstream repository should pass it explicitly, for example `gh pr list -R kleverhq/wavepeek`. Browser-based PR creation remains supported and must not require GitHub CLI authentication.

Before proposing substantial work, check GitHub Milestones and open GitHub Issues. If the change needs product or maintainer discussion, open or reference an issue before starting a PR.
