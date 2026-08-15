# Docs helper tools

This directory contains helper scripts and tests for the GitHub Pages documentation pipeline. The stable entrypoints are the root `justfile` recipes and `.github/workflows/docs.yml`; call these scripts directly only when developing or debugging the helpers.

## Helpers

- `prepare_mkdocs.py` validates an extracted skill and stages cumulative versioned documentation with Mike navigation.
- `prepare_playground.py` stages the framework-free current Playground, generated WASM, and verified bundled demo for a root Material build.
- `check_playground.py` checks the composed `/wavepeek/` and `/wavepeek/latest/` preview, compares browser and native behavior, and covers terminal interaction, install-prompt copying, local-file privacy, worker recovery, shared theme state, and responsive layout.
- `publish_docs.py` owns local `check`, no-token `stage-deploy`, and credentialed `push-staged`. It accumulates documentation versions, replaces root Playground/installers only when promoting latest, and exports the verified `gh-pages` tree to the Pages artifact.
- `check_deploy.py` verifies the current Playground/demo, versioned documentation, CORS, and optional GitHub Pages API state after publication.
- `workflow_docs.py` keeps GitHub Actions glue testable: dispatch validation, release preflight, and workflow environment translation for stage/push jobs.

## Tests

Run helper tests with:

    python3 -B -m unittest discover -s tools/docs -p "test_*.py"

`just test-aux`, `just check`, and `just ci` include these tests or the docs-site check through the repository quality gates.
