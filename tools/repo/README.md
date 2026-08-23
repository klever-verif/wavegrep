# Repository Tools

This group owns repository-maintenance helpers that are not part of the public `wavepeek` CLI.

Normal statistics entrypoint:

    python3 -B tools/repo/repo_stats.py

`git-hook` is the reviewed source for host `pre-commit` and `commit-msg` dispatch. Install worktree-local copies with `./dev --install-hooks`; do not execute the tracked source directly.

`repo_stats.py` prints stable line-count categories for source, tests, collateral helper code, JSON fixtures, and docs. `Total code` includes source, tests, benchmarks, and tools. `Total lines` adds JSON fixtures and Markdown docs while excluding disposable, generated, and branch-local tracking content.
