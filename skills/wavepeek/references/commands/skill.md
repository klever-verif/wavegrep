# Skill command

Use `wavepeek skill <DIRECTORY>` to extract the complete, version-matched Wavepeek skill package into a new or empty directory.

For exact syntax and argument details, run `wavepeek help skill`.

The extracted package contains:

- `SKILL.md`, the agent entrypoint;
- `references/`, the offline command, workflow, troubleshooting, and semantic guidance;
- `examples/`, reserved for concrete examples and currently empty;
- `manifest.json`, package and bundle-format provenance.

The command never merges with or overwrites existing files. Choose a missing or empty destination directory, then point the agent at `SKILL.md`. The command is human-oriented and does not support `--json` or `--jsonl`.
