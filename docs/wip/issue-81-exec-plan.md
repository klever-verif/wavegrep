# Accept canonical signal paths under a selected scope

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

A caller that discovers a canonical signal path in JSON should be able to copy that path into `value`, `change`, `property`, or `extract generic` while retaining `--scope`. After this change, `cpu.valid` and `top.cpu.valid` both resolve to the canonical waveform signal `top.cpu.valid` under `--scope top`, and one request may mix those forms. A canonical path outside the selected scope remains unresolved because every other token is interpreted relative to the selected scope.

## Non-Goals

Protocol-specific `extract ahb`, `extract apb`, `extract atb`, `extract axi`, and `extract axistream` `--map` behavior does not change. No ambiguity warning is added when an input such as `other.sig` could also name a canonical path outside the scope; under `--scope top`, it unconditionally means relative path `top.other.sig`. No output schema changes are required.

## Progress

- [x] (2026-08-12) Read issue #81, related issue #80, repository guidance, resolver call sites, tests, CLI help, and packaged documentation.
- [x] (2026-08-12) Confirm scope decisions with the maintainer: exclude protocol `--map`, prefer relative interpretation, and omit an ambiguity diagnostic.
- [x] (2026-08-13) Changed the shared resolver and focused unit/integration tests.
- [x] (2026-08-13) Updated CLI help and packaged scoped-naming documentation.
- [x] (2026-08-13) Ran focused unit, CLI, fixture-contract, and help suites plus `./dev just ci`; all passed.
- [x] (2026-08-13) Committed implementation and documentation as `3798c93`.
- [x] (2026-08-13) Ran Luna Max correctness/tests, docs/help, and architecture/performance/simplicity lanes; fixed docs/help findings and revalidated with `cli_contract` plus `just check`.
- [ ] Run Terra High focused review lanes over the consolidated diff, fix findings, and revalidate.
- [ ] Run a Sol High control review, fix findings, and run final gates.
- [ ] Remove this branch-local plan, commit cleanup, push, and open the pull request.

## Surprises & Discoveries

- Observation: issue #80 already consolidated all generic scoped signal references behind one helper.
  Evidence: `src/engine/mod.rs::scoped_signal_path` is called by signal lists, expression binding, and generic extraction payload resolution.

- Observation: protocol-specific `--map` implementations have separate dotted-name rejection and are intentionally outside issue #81.
  Evidence: `src/engine/ahb.rs`, `apb.rs`, `atb.rs`, `axi.rs`, and `axistream.rs` each own `explicit_mappings` logic.

- Observation: once canonical descendants are accepted, scoped path construction cannot fail and its `Option<String>` return plus synthetic rejection diagnostics become dead code.
  Evidence: changing `scoped_signal_path` to return `String` removed four unreachable `ok_or_else` branches and the now-unused expression diagnostic helper.

- Observation: an old command-runtime negative fixture encoded the former canonical-path rejection.
  Evidence: `property_scoped_dotted_on_expr` became a successful command and its obsolete manifest row and snapshot were removed.

- Observation: broad naming prose can accidentally imply that protocol-specific `extract --map` accepts canonical paths.
  Evidence: Luna Max docs review found unqualified wording in `skills/wavepeek/SKILL.md` and `command-model.md`; both now explicitly preserve scope-relative protocol mappings.

## Decision Log

- Decision: Preserve the existing shared resolver and change only its active-scope-prefix branch from rejection to canonical pass-through.
  Rationale: This is the smallest root-cause fix and automatically covers every acceptance-criteria surface without command-specific branches.
  Date/Author: 2026-08-12 / pi

- Decision: Under `--scope top`, a token not starting with the exact `top.` boundary is always relative, even if the same token exists canonically outside `top`.
  Rationale: This is deterministic, keeps scope as the namespace root, and matches maintainer direction. Looking up both interpretations would add complexity and inconsistent diagnostics.
  Date/Author: 2026-08-12 / maintainer and pi

- Decision: Do not change protocol-specific extraction mappings or add an ambiguity diagnostic.
  Rationale: `--map` is not in the issue acceptance criteria, and a cross-surface warning would require waveform-aware resolution plumbing for little benefit.
  Date/Author: 2026-08-12 / maintainer and pi

- Decision: Qualify top-level skill and command-model prose to generic waveform queries and retain surface-specific nouns in help.
  Rationale: Luna Max review identified a real risk of promising unsupported protocol mapping behavior and weak help-contract coverage; precise wording is the smallest fix.
  Date/Author: 2026-08-13 / pi

## Outcomes & Retrospective

The shared implementation and public contract updates are complete and committed. Focused validation and full CI pass; review waves, final cleanup, and PR creation remain.

## Context and Orientation

WavePeek is a Rust command-line waveform inspector. A canonical signal path is the complete dot-separated path from the waveform root, such as `top.cpu.valid`. A relative signal path is interpreted beneath the value supplied to `--scope`; under `--scope top`, `cpu.valid` names the same signal.

`src/engine/mod.rs::scoped_signal_path` is the shared lexical resolver. Without a scope it preserves the input. With a scope it currently prefixes relative names but returns `None` for an input already beginning with the exact selected-scope prefix. `src/engine/value.rs` and `src/engine/change.rs` use it for `--signals`; `src/engine/expr_runtime.rs` uses it for names in `--on`, `--eval`, and `--when`; `src/engine/extract.rs` uses it for generic `--payload` names. The waveform layer then resolves the resulting canonical path.

Integration tests live in `tests/value_cli.rs`, `tests/change_cli.rs`, `tests/property_cli.rs`, and `tests/extract_generic_cli.rs`. CLI wording lives in `src/cli/`, with exact help assertions in `tests/cli_contract.rs`. Public scoped naming rules live in `skills/wavepeek/SKILL.md` and Markdown references under `skills/wavepeek/references/`.

## Open Questions

There are no open product questions. Review findings may require narrow follow-up decisions, which must be recorded here.

## Plan of Work

First, change `src/engine/mod.rs::scoped_signal_path` so an input beginning with the selected scope followed by a dot is returned unchanged. Keep the boundary check so `topology.valid` under `--scope top` remains relative and resolves to `top.topology.valid`. Update the adjacent unit test to prove no-scope, short-relative, descendant-relative, canonical-inside, and prefix-lookalike behavior.

Next, replace old-contract integration assertions with public behavior tests. Cover canonical paths inside scope and mixed canonical/relative references across `--signals`, expression surfaces (`--on`, `--eval`, `--when`), and generic `--payload`. Keep or add a rejection proving an outside canonical spelling is interpreted relative to the active scope and therefore cannot escape it. Prefer existing fixtures and existing test functions rather than new fixture infrastructure.

Then update concise CLI help and packaged references. State that scoped references may be relative or canonical paths inside the selected scope, may be mixed, and that other names remain relative to the selected namespace. Remove stale instructions that prohibit canonical scoped inputs. Do not alter protocol-specific `--map` documentation.

Validate focused suites, then the repository CI gate. Commit the complete behavior slice. Run three parallel read-only review lanes for correctness/tests, docs/help contracts, and architecture/performance using Luna Max. Apply substantive findings and commit. Repeat the same areas with Terra High. Finally run one fresh Sol High control review over the complete branch diff, address substantive findings, run final gates, remove this plan, push, and open a pull request against `dev3`.

### Concrete Steps

Run all commands from the repository root `/home/esynr3z/projects/wavepeek/.worktrees/wavepeek/81`.

Start the worktree container and install hooks before the first commit:

    ./dev true
    ./dev --install-hooks

During implementation, format and run focused tests:

    ./dev cargo fmt --all
    ./dev cargo test --test value_cli --test change_cli --test property_cli --test extract_generic_cli --test cli_contract

Before review and again after substantive review fixes, run:

    ./dev just ci

Inspect the final branch and push it:

    git status --short
    git log --oneline origin/dev3..HEAD
    git push -u origin dev3-81/canonical-scope-paths

Open the pull request:

    gh pr create --base dev3 --head dev3-81/canonical-scope-paths --title "fix(cli): accept canonical paths with scope" --body-file <prepared-file>

Expected focused and CI transcripts end successfully with no failed tests or quality gates. The exact test count may change while tests are edited.

### Validation and Acceptance

The unit resolver test must prove that `scoped_signal_path("top.cpu.valid", Some("top"))` yields `top.cpu.valid`, while `scoped_signal_path("topology.valid", Some("top"))` yields `top.topology.valid`.

Integration tests must demonstrate all issue surfaces. With `--scope top`, relative and canonical names inside `top` resolve successfully and can be mixed in one request. This applies to `value` and `change` signal lists, `change` and `property` event/evaluation expressions, and generic extraction event, predicate, and payload references. A spelling such as `outside.sig` is interpreted as `top.outside.sig`; if no such descendant exists, the command fails rather than reaching canonical `outside.sig`.

`wavepeek value --help`, related command help, `skills/wavepeek/references/command-model.md`, and `skills/wavepeek/references/scoped-vs-canonical-names.md` must describe accepted relative and in-scope canonical forms without promising protocol `--map` changes.

The final `./dev just ci` must pass. Review completes only after Luna Max lanes, matching Terra High lanes, and the Sol High control pass return no unresolved substantive findings.

### Idempotence and Recovery

Container startup, hook installation, formatting, tests, and quality gates are safe to repeat. Generated waveform files remain ignored. If a review fix invalidates a prior test result, rerun the focused suite and `just ci`. Do not bypass hooks. Do not delete unrelated files under `tmp/` or another branch-local WIP artifact.

### Artifacts and Notes

The intended resolver behavior is equivalent to:

    no scope                      -> preserve input
    input starts with "scope."    -> preserve canonical input
    every other input with scope  -> prefix "scope."

This exact dot boundary prevents `topology.valid` from being mistaken for a canonical descendant of `top`.

### Interfaces and Dependencies

Keep the existing internal signature in `src/engine/mod.rs`:

    pub(crate) fn scoped_signal_path(name: &str, scope: Option<&str>) -> Option<String>

Do not add dependencies, new public types, or new resolution layers. Existing `clap`, waveform backends, expression host, and error contracts remain unchanged.

Revision note (2026-08-13): Recorded implementation, focused-test, and CI completion; documented the infallible resolver simplification, removed obsolete negative-fixture behavior, and corrected the container startup command after observing the repository entrypoint contract.

Revision note (2026-08-13): Recorded Luna Max review completion, its docs/help findings, the resulting protocol-scope clarification and help-contract coverage, and successful revalidation.
