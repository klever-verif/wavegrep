# Public Reference Topic Guidance

## Source of Truth

- Cross-cutting command semantics: `command-model.md`
- Machine output, diagnostics, and exit behavior: `machine-output.md`
- Expression language syntax and semantics: `expression-language.md`
- Machine output behavior: `machine-output.md` and direct runtime tests under `../../../tests/`
- Topic metadata and docs style rules: `../../dev/style.md`
- Exact command reference: `../../../src/cli/`, `wavepeek --help`, `wavepeek help <command-path...>`, and `wavepeek docs --help`

## Local Guidance

Keep these topics focused on stable behavior that code and generated help do not explain clearly enough. Avoid release planning, maintainer process, or speculative future syntax here.
