# Skill development guide

## Scope and model

This document is the source of truth for developing the bundled skill under `../skills/`.

The skill is primarily a router and map of the bundle. `SKILL.md` should direct agents to the right reference instead of duplicating detailed command contracts. Aim for the examples to provide roughly 80% of the practical understanding of how Wavepeek works.

## Package sources

- `../skills/wavepeek/` is the canonical package source for the embedded skill.
- `../skills/wavepeek/SKILL.md` is the entry point, router, and bundle map.
- `../skills/wavepeek/references/docs.json` defines the documentation tree consumed by the renderer.
- `../skills/wavepeek/references/cli-reference.md` is generated. Do not edit it by hand.

Do not duplicate package content in Rust or edit generated copies.

## Content boundaries

- Keep reference articles concise and focused on one subject.
- Keep usage articles practical and task-oriented.
- Put detailed semantics in concept and reference articles, not in usage articles.
- Avoid repeating the same explanation across `SKILL.md`, references, examples, and usage guides.
- Keep examples simple, readable, and easy to adapt.

## Example policy

Examples intentionally reduce complexity. They may use simple error handling and do not need production-grade validation or defensive programming unless the example's documented contract requires it.

Simplify error handling and presentation, not command syntax, output contracts, protocol semantics, or safety rules. An example may be incomplete by design, but it must not teach an incorrect command, output shape, or protocol behavior.

## Consistency requirements

- Keep `docs.json` consistent with the files beside it. Update its entries whenever a reference is added, renamed, moved, or removed.
- Keep links from `SKILL.md`, `docs.json`, and all reference articles consistent with the actual package tree.
- `references/index.md` and `references/quickstart.md` contain many links to neighboring articles. Check them whenever the reference tree changes.
- Before removing or renaming a skill file, search the repository for its path and update every affected reference.
- Keep examples and their README files consistent with the machine-output and command contracts.

## Verification

- Run the relevant auxiliary tests after changing examples or scripts.
- Run `just check` before handing off changes that affect the package, navigation, generated references, or documentation links.
- When changing the package layout, verify skill extraction and the rendered documentation tree.
