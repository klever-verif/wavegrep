# DO NOT DELETE THIS FILE

This breadcrumb keeps this otherwise-empty directory tracked. Do not remove it during WIP cleanup.

# WIP Tracker Guidance

## Scope

This directory is for branch-local tracked artifacts that must survive across agent sessions, such as active execution plans or reviewed investigation notes.

## Local Guidance

- Use repository-root `../../tmp/` for ignored scratch files, logs, and disposable outputs.
- Use this directory only when the artifact should be reviewed, committed, and available after a fresh checkout of the branch.
- Remove branch-local WIP artifacts before merging to the default branch unless a maintainer explicitly wants to keep them for handoff.
- Do not delete another agent's WIP files unless the current branch cleanup clearly owns them.
