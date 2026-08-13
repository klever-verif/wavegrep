# Report malformed expression literals at their source

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

WavePeek currently blames a balanced opening parenthesis when a later integral literal is malformed. After this change, users who write C-style hexadecimal text such as `0x10` or omit the apostrophe in a sized SystemVerilog literal such as `64h10` will receive a literal-specific error whose numeric span and caret range cover the complete malformed token. A truly unclosed parenthesis will continue to report `EXPR-PARSE-LOGICAL-UNMATCHED-OPEN`.

The behavior is visible by running `wavepeek property` against `tests/fixtures/hand/change_property_events.vcd` with `--eval 'clk == (clk >= 64h10)'`: stderr must identify `64h10`, suggest `64'h10`, underline all five characters, preserve the `fatal: expr:` transport, and exit with status 1.

## Non-Goals

This work does not add C-style hexadecimal syntax, broaden the supported expression language, change JSON or JSONL success envelopes, alter fatal exit behavior, introduce a new diagnostic abstraction, or redesign source-location reporting into line and column coordinates.

## Progress

- [x] (2026-08-13 18:14Z) Read issue #103, repository guidance, expression lexer/parser/renderer paths, tests, and packaged skill.
- [x] (2026-08-13 18:14Z) Install this worktree's host Git hooks with `./dev --install-hooks`.
- [x] (2026-08-13 18:22Z) Add focused regression coverage for both malformed literals, parenthesis classification, caret rendering, CLI transport, and generic source files.
- [x] (2026-08-13 18:22Z) Implement the shared lexer, parser, and renderer corrections without new interfaces or dependencies.
- [x] (2026-08-13 18:22Z) Add the compact literal reminder to the bundled skill and update affected snapshots.
- [ ] Run focused tests, `./dev just ci`, and `./dev just check`; commit logical milestones. Completed: focused tests and `./dev just ci`; remaining: commit and `./dev just check` before handoff.
- [ ] Run two focused review waves and one independent control review, resolving substantive findings. Completed: Luna Max and Terra High correctness, diagnostics, and docs/simplicity lanes; fixed Luna's stale plan caret example and Terra's broad `64hname` classification. Remaining: Sol High control.
- [ ] Remove this branch-local plan, run final checks, push the branch, and open a pull request closing issue #103.

## Surprises & Discoveries

- Observation: All requested command surfaces already converge on the same logical lexer and parser through `src/engine/expr_runtime.rs`; generic source-file expressions also use that route.
  Evidence: `change`, `property`, and generic `extract` binding call the shared `bind_waveform_event_expr` or `bind_waveform_logical_expr` helpers.

- Observation: The literal diagnostic code already exists.
  Evidence: `src/expr/lexer.rs` and `src/expr/sema.rs` already emit `EXPR-PARSE-LOGICAL-LITERAL`, so no new code family is needed.

- Observation: The shared renderer change mechanically affects every stored expression diagnostic, including zero-width runtime spans.
  Evidence: Twelve existing snapshots gained only the expected caret line; `real'(xbus)` renders one caret for its existing `0..0` span.

- Observation: `./dev` writes its devcontainer startup record to stdout, so it is not suitable for manually proving the binary's stdout is empty.
  Evidence: the exact issue command returned status 1 and the expected stderr after a devcontainer startup line; integration tests invoking the binary directly prove empty stdout.

- Observation: A bare `h` prefix is also an ordinary identifier boundary unless a based digit follows it.
  Evidence: Terra review identified that the first detector classified `64hname` as a malformed literal; the detector now checks the next character and a unit regression preserves `64` plus identifier `hname`.

## Decision Log

- Decision: Detect only the two malformed families required by the issue: a numeric token beginning with `0x`/`0X`, and decimal size digits immediately followed by `h`/`H` plus based digits without an apostrophe.
  Rationale: Narrow detection preserves existing treatment of ordinary trailing identifiers and avoids pretending to support speculative malformed forms.
  Date/Author: 2026-08-13 / Pi

- Decision: Keep `EXPR-PARSE-LOGICAL-UNMATCHED-OPEN` only when the parser reaches end of input while waiting for `)`; use the existing expected-token diagnostic for any other token.
  Rationale: This directly separates a missing close from a balanced expression containing a later syntax error without adding parser state.
  Date/Author: 2026-08-13 / Pi

- Decision: Extend the existing renderer directly rather than adding a source-map or rendering layer.
  Rationale: Issue #103 requires a caret range and retained byte span, not general multiline diagnostics. The existing `ExprDiagnostic::render` is the shared source of user-visible text.
  Date/Author: 2026-08-13 / Pi

## Outcomes & Retrospective

Implementation is not complete. The intended result is one shared correction rather than per-command guards, with regression evidence across direct expression parsing, CLI transport, and generic source-file loading.

## Context and Orientation

WavePeek is a Rust command-line program. `src/expr/lexer.rs` converts logical expression source into tokens. Its `LogicalLexer::lex_numeric_literal` currently stops after decimal digits, so `0x10` becomes decimal `0` followed by identifier `x10`, and `64h10` becomes decimal `64` followed by identifier `h10`. `src/expr/parser.rs` turns those tokens into an expression tree. In `LogicalParser::parse_primary_expr`, a parenthesized expression currently reports the opening parenthesis as unmatched whenever the token after the parsed inner expression is not `)`, even if that token is not end of input.

An `ExprDiagnostic` in `src/expr/diagnostic.rs` carries a layer, stable code, message, `primary_span`, and notes. A span uses byte offsets with an exclusive end. `ExprDiagnostic::render` produces the text wrapped by `src/engine/expr_runtime.rs` in `WavepeekError::Expr`; `src/main.rs` then prints the unchanged `fatal: expr: ...` process error and returns status 1.

The expression command surfaces are `change --on`, `property --on`, `property --eval`, `extract generic --on`, and `extract generic --when`. Generic extraction source JSON reaches the same binders in `src/engine/extract.rs`, so shared lexer/parser changes cover it. Structured expression cases live in `tests/fixtures/expr/`; integration suites invoke real CLI commands under `tests/`; Insta snapshots under `tests/snapshots/` preserve rendered diagnostic text only where that text is a contract. The canonical packaged agent skill is `skills/wavepeek/SKILL.md`.

## Open Questions

There are no blocking product questions. During implementation, token-boundary behavior will be constrained by focused tests so malformed literal recognition does not swallow following operators or unrelated identifiers.

## Plan of Work

First add the smallest regression cases around the shared behavior. Lexer or expression-manifest tests will assert `EXPR-PARSE-LOGICAL-LITERAL`, the complete primary span for `0x...` and `64h10`, and the required notes. Parser coverage will distinguish an unexpected token after a balanced inner expression from end of input after a genuinely unclosed parenthesis. Renderer coverage will snapshot or directly assert the numeric span and complete caret range. Existing integration conventions will cover representative direct CLI surfaces and generic source-file loading without duplicating every internal assertion at every call site.

Then edit `LogicalLexer::lex_numeric_literal` in `src/expr/lexer.rs`. Immediately after consuming the decimal prefix, recognize `0x` or `0X` and consume the rest of that contiguous alphanumeric/underscore token before returning `EXPR-PARSE-LOGICAL-LITERAL` with a note that names accepted forms `'h10` and `8'h10`. Recognize a decimal size followed immediately by `h` or `H`, consume the same token boundary, and return the same code with message `malformed sized integral literal` and a note to insert an apostrophe, illustrated by `64'h10`. Existing valid decimal, real, exponent, and apostrophe-based branches must remain unchanged.

Next edit the parenthesis branch in `LogicalParser::parse_primary_expr` in `src/expr/parser.rs`. If the current token after parsing the inner expression is EOF, retain the existing unmatched-open code and opening-parenthesis span. If it is another token, emit the existing `EXPR-PARSE-LOGICAL-EXPECTED` code on that current token with a note that `)` was expected. Malformed literals should normally fail earlier in the lexer, but this parser correction prevents unrelated later tokens from being misclassified.

Extend `ExprDiagnostic::render` in `src/expr/diagnostic.rs` with a line directly after `source:` that uses spaces followed by one or more carets covering `primary_span`. Clamp malformed or zero-width spans safely to source bounds and render at least one caret. Preserve the existing numeric `--> span start..end` line. Keep the implementation local and standard-library-only.

Add the requested reminder to `skills/wavepeek/SKILL.md`: SystemVerilog-style hexadecimal examples `64'h10` and `128'h0011...` are valid forms while `0x...` and `64h10` are invalid. Update only snapshots affected by the shared renderer and add a malformed-literal snapshot only if rendered wording needs explicit contract coverage beyond direct assertions.

After focused tests pass, run full project gates. Commit the functional slice with a conventional `fix(expr): ...` commit. Run review wave one using parallel read-only Luna reviewers at maximum reasoning, divided into correctness/testing, diagnostics/output, and docs/complexity lanes. Every lane must apply KISS, YAGNI, and ponytail-review principles. Resolve findings and commit fixes. Repeat the same lanes with fresh Terra reviewers at high reasoning. Resolve findings and commit fixes. Run a fresh Sol reviewer at high reasoning over the consolidated diff as the control pass, resolve any substantive finding, remove this WIP plan, rerun final gates, push, and open a GitHub pull request that closes #103.

### Concrete Steps

Run all repository commands from `/home/esynr3z/projects/wavepeek/.worktrees/wavepeek/103`. Cargo and quality commands must go through the existing devcontainer entrypoint.

    ./dev just test

The focused implementation loop may use exact Cargo test filters through `./dev cargo test ...`. After implementation:

    ./dev just ci
    ./dev just check

Both commands must exit successfully. Exercise the reported case with the built CLI or an integration assertion and expect stderr shaped like:

    fatal: expr: parse:EXPR-PARSE-LOGICAL-LITERAL: malformed sized integral literal
    --> span 15..20
    source: clk == (clk >= 64h10)
    <five carets under 64h10>
    note: insert an apostrophe: 64'h10

The exact spacing follows the shared renderer, while the full token span, five-caret range, note, exit status 1, and `fatal: expr:` prefix are mandatory.

Before each commit, inspect `git diff --check` and the staged diff. Hooks must run normally. Before review, record `git diff origin/main...HEAD`, test results, and the commit range in reviewer prompts. Use three parallel lanes in each requested wave and one fresh control reviewer after both waves.

### Validation and Acceptance

The shared expression tests must prove `0x10` and a longer `0x...` token report `EXPR-PARSE-LOGICAL-LITERAL` over the complete token and mention SystemVerilog forms. They must prove `64h10` reports the same code over all five bytes and suggests `64'h10`. A balanced expression containing a different unexpected token must not report unmatched open, while `clk == (clk >= 10` must still report `EXPR-PARSE-LOGICAL-UNMATCHED-OPEN` on `(`.

Rendered output must retain `--> span start..end`, display the source, and add a caret or caret range matching the complete primary span. Existing diagnostic snapshots will ensure this behavior applies to parse, semantic, runtime, and fatal CLI diagnostics. Representative integration tests must show the malformed forms are consistent through event and logical command options and through generic source JSON. Existing error tests plus the new CLI assertions must preserve empty stdout, status 1, and the `fatal: expr:` prefix.

`skills/wavepeek/SKILL.md` must contain both `64'h10` and `128'h0011...` and explicitly reject `0x...` and `64h10`. `./dev just ci` and `./dev just check` must pass. Required reviews must complete with no unresolved substantive findings.

### Idempotence and Recovery

Test and formatting commands are safe to repeat. Insta may create `.snap.new` files when rendered output changes; inspect and accept only expected caret additions, then remove stale `.snap.new` files through the normal Insta workflow rather than deleting unrelated files. Do not clean repository-root `tmp/` globally. If a commit hook fails, keep the container running, fix the reported cause, and retry without bypassing hooks. The WIP plan is owned by this branch and must be removed before the final pull request.

### Artifacts and Notes

Issue #103 supplies the repository fixture command:

    wavepeek property --waves tests/fixtures/hand/change_property_events.vcd \
      --scope top --on '*' --sample-mode native \
      --eval 'clk == (clk >= 64h10)' --capture match

Before the fix this incorrectly reports:

    EXPR-PARSE-LOGICAL-UNMATCHED-OPEN
    --> span 7..8

The implementation must instead identify bytes 15 through 20 in this shortened expression and underline `64h10`.

### Interfaces and Dependencies

No public Rust interface or dependency is added. The existing `LogicalLexer::lex_numeric_literal`, `LogicalParser::parse_primary_expr`, and `ExprDiagnostic::render(&self, source: &str) -> String` remain the ownership points. Existing `ExprDiagnostic`, `Span`, `DiagnosticLayer`, `WavepeekError::Expr`, Insta, and current integration-test helpers are reused.

Plan revision note: 2026-08-13 — created the initial self-contained execution plan after tracing issue #103 through the shared expression and CLI paths.

Plan revision note: 2026-08-13 — recorded completed implementation, regression coverage, snapshot effects, and successful focused plus CI validation.

Plan revision note: 2026-08-13 — recorded the first Luna Max review wave and corrected its only finding, a stale hand-aligned caret example in this plan.

Plan revision note: 2026-08-13 — recorded the Terra High review wave and narrowed missing-apostrophe recognition to require a following hexadecimal based digit.
