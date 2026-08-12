# Quality Gates

`wavepeek` quality gates are owned by the root `justfile`. Run them from the host through `./dev`; most recipes intentionally fail outside the shared development container so local and CI results do not drift.

## Standard Gates

Use `./dev just check` before handing off local work. It runs formatting checks, clippy, schema contract validation, GitHub Actions linting, docs-site generation checks, `cargo check`, commit-message validation, and the FSDB build gate when Verdi is available.

Use `./dev just ci` when tests should be included. It runs formatting, lint, schema and GitHub Actions checks, auxiliary Python unit tests, the source coverage gate, `cargo check`, docs-site generation checks, and the FSDB regression path when Verdi is available. Unlike `check`, it does not validate the current commit message; `test-aux` still validates the FSDB benchmark catalog. The Rust test execution used by CI is the `cargo llvm-cov` run behind `just coverage-src-check`; use `just test` when you want an explicit non-coverage Rust test pass while iterating. `just test` also runs the FSDB regression path when Verdi is available.

Use `./dev just pre-commit` to run every Pre-commit check across the repository. Host hooks are installed explicitly by `./dev --install-hooks` and must not be bypassed unless a maintainer asks. They require the current worktree's container to be running because they invoke only `./dev --exec-only`. The hook path inherits conditional FSDB lint, test, and benchmark smoke behavior from the `just` recipes when Verdi is available, but the FSDB pre-commit benchmark smoke prepares only its small filtered RTL fixture set instead of the full FSDB benchmark catalog. Benchmark smoke validates that selected commands still run and emit artifacts; it does not compare against committed baselines.

## Focused Recipes

- `just format` and `just format-check` run rustfmt and the justfile formatter.
- `just lint` runs clippy with warnings denied, plus feature-enabled FSDB clippy when Verdi is available; `just lint-fix` applies safe clippy suggestions when useful.
- `just check-build` runs `cargo check`.
- `just check-schema` validates generated schema snapshots (`schema/output.json`, `schema/stream.json`, `schema/input.json`, and `schema/catalog.json`) against the Rust contract generator and runtime schema output.
- `just check-actions` runs `actionlint` for `.github/workflows/`.
- `just docs-site-build` exports embedded public docs into `tmp/docs-site/`, generates a MkDocs config, and builds the site in strict mode.
- `just docs-site-check` runs the full local docs publication check, including root Pages artifacts, without touching `gh-pages`.
- `just bench-gate`, `just bench-capture`, and `just bench-compare` support manual performance review and are not part of the standard pre-handoff gates.
- `just coverage-src` reports source coverage without enforcing thresholds.
- `just coverage-src-check` enforces the current `90%` minimum on lines, regions, and functions for `src/**`.
- `just check-commit [message-path]` runs Commitizen against the supplied file or Git's standard commit-message file by default.

## Optional FSDB Gates

The optional `fsdb` Cargo feature requires a local Synopsys Verdi FSDB Reader SDK. `just lint`, `just check`, `just test`, `just ci`, and the benchmark pre-commit smoke probe the local environment: when `tools/fsdb/check_fsdb_env.py` validates a usable SDK, they run the relevant FSDB gates; when Verdi is absent, they print a skip message and continue.

Focused FSDB recipes include `just check-fsdb-env`, `just lint-fsdb`, `just check-fsdb-build`, `just prepare-fsdb-fixtures`, `just prepare-fsdb-test-fixtures`, `just test-fsdb`, and `just bench-e2e-fsdb-smoke-commit`. The detailed FSDB gate and SDK contract lives in `fsdb.md`.

## Interpreting Failures

Schema freshness failures usually mean the Rust contract schema output changed; regenerate with `just update-schema` only if the schema change is intended. Coverage failures should be fixed by adding meaningful tests, not by lowering thresholds. If `just check-commit` fails before any commit exists because `.git/COMMIT_EDITMSG` is missing or stale, create the commit normally and let the commit-msg hook validate the actual message.

Keep stdout and stderr from project tools deterministic. If a gate reports unstable output, treat that as a product-contract problem rather than noisy tooling.
