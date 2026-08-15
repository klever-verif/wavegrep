# Style and Contract Conventions

This document covers maintainer conventions for Rust code, CLI behavior, deterministic output, and the packaged skill. Public user-visible semantics live under `skills/wavepeek/references/`; do not duplicate that reference surface here.

## Rust Style

Use rustfmt. Avoid manual alignment or stylistic churn that fights the formatter. Prefer explicit control flow over clever one-liners when clarity or error handling matters.

Use explicit imports instead of glob imports except in `mod.rs` where it materially reduces noise. Keep imports in the usual Rust order: standard library, external crates, then crate modules. Clippy runs with warnings denied, so unused imports are gate failures, not decorative lint confetti.

Use `snake_case` for modules, functions, and locals; `PascalCase` for types, traits, and enums; and `SCREAMING_SNAKE_CASE` for constants. CLI flags should be long, self-documenting, and kebab-case through clap.

Prefer borrowing at API boundaries. Use owned `String` and `Vec` when ownership is required, not because cloning felt easier. Avoid `Box<dyn Error>` in core paths; prefer typed errors with `thiserror`.

## Error Handling

No panics in production paths. Avoid `unwrap()` and `expect()` except for true programmer bugs that are unreachable in normal operation. Human errors go to stderr. Machine-mode fatal errors and command output go to stdout according to `skills/wavepeek/references/machine-output.md`.

Preserve the stable human process-level failure shape:

    fatal: <category>: <message>

Successful commands may emit non-fatal diagnostics. Human diagnostics use `warning[WPK-W####]: <message>` or `error[WPK-E####]: <message>`, and JSON diagnostics use typed objects in the success envelope.

Also preserve exit-code behavior. Exit code `0` is success, `1` is user-facing argument or query failures, and `2` is file open/parse failure.

## Deterministic Output

Identical inputs must produce identical outputs. Sort user-facing collections deterministically, avoid timestamps and random IDs, and never rely on hash-map iteration order. Default result sets must stay bounded with flags such as `--max` and `--max-depth`.

## CLI Design Constraints

Waveform-inspection commands use named flags for primary inputs. The waveform file flag is always `--waves`. Default output is human-readable; `--json` enables the strict JSON envelope documented in `skills/wavepeek/references/machine-output.md`. Time values require explicit units; reject bare numbers.

Help must remain layered and standalone: `wavepeek` with no args aliases compact help, `-h` stays compact, `--help` stays detailed, and `wavepeek help <command-path...>` aliases long nested help. `wavepeek skill <DIRECTORY>` extracts the complete packaged skill into a new or empty directory.

## Packaged Skill Maintenance

The canonical package source lives under `skills/wavepeek/`. Keep `SKILL.md` concise, keep narrative guidance as flat Markdown files under `references/`, and keep internal links relative so they resolve in an extracted package. Exact syntax, flags, defaults, and required arguments belong in generated CLI help.

`skills/wavepeek/references/docs.json` is the source of truth for website navigation labels, groups, and order. Every reference Markdown file must appear exactly once. The extracted package is rooted at `SKILL.md` and includes `references/`, `examples/`, and `manifest.json`. Keep `examples/` empty until a concrete example is required.
