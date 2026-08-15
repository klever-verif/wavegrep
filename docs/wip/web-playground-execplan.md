# Ship the current WavePeek browser Playground

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document must be maintained in accordance with the `exec-plan` skill.

## Purpose / Big Picture

After this work, visiting the WavePeek GitHub Pages root opens one browser-only Playground for the latest stable WavePeek release. A user can run the real WavePeek CLI parser and Rust engine against the bundled `scr1_axi.fst` demo or one local VCD/FST without uploading the waveform. Versioned Material documentation remains available through Mike, but old documentation versions always link to the one current Playground and do not retain old Playground builds.

The result is observable locally by running `./dev just playground-serve`, opening the printed URL, executing `wavepeek info --waves scr1_axi.fst`, selecting a local waveform, navigating to documentation, and opening the same demo in Surfer. The final local server remains running for maintainer inspection; no branch is pushed.

## Non-Goals

This work does not add FSDB, `wavepeek skill`, extraction `--source <FILE>`, persistence across reloads, multiple active uploads, waveform editing, a terminal emulator, a JavaScript command reimplementation, archived Playground versions, or historical demo copies. It does not add a frontend framework, Node.js, npm, a package manager, telemetry, a backend, or a reusable v3 reset mechanism.

## Progress

- [x] (2026-08-15 10:42Z) Read the proposal, repository guidance, native CLI path, waveform backend, Material/Mike publication path, and quality workflow.
- [x] (2026-08-15 10:42Z) Confirmed the existing crate compiles for `wasm32-unknown-unknown` and Wellen exposes byte-reader APIs for both ordinary and streaming VCD/FST access.
- [x] (2026-08-15 10:42Z) Confirmed `tmp/scr1_axi.fst` is 3,648,879 bytes with SHA-256 `aad73e9b0d2b244b67a96b254371ff29a2ac2e54077176376f6361570789e884` and exercises WavePeek's AXI-facing hierarchy.
- [x] (2026-08-15 10:56Z) Proved `wavepeek info --waves scr1_axi.fst` through generated WASM in headless Chrome; status was zero and the expected 1 ps bounds were returned.
- [x] (2026-08-15 10:56Z) Added explicit argv/output writers, invocation-scoped waveform bytes, byte-backed Wellen parsing/streaming, truthful browser help, and the minimal WASM binding while preserving focused native tests.
- [x] (2026-08-15 11:28Z) Built the framework-free worker-based Playground, committed demo source, editable synchronized CLI, local-file flow, Surfer link, history, Stop recovery, and responsive Material page.
- [x] (2026-08-15 11:28Z) Changed MkDocs/Mike staging so only documentation accumulates and promoted releases replace the root Playground while historical repairs preserve it.
- [x] (2026-08-15 11:28Z) Added pinned WASM/Playwright tooling, native-browser parity and privacy checks, publication tests, deployment asset/CORS checks, and a pre-v3 tree guard.
- [x] (2026-08-15 11:31Z) Updated maintainer architecture, environment, automation, testing, quality, release, and tooling documentation.
- [x] (2026-08-15 12:42Z) Passed focused checks, post-review `just ci`, and post-review `just check`. The manual benchmark gate is explicitly deferred until the maintainer selects suitable benchmark conditions.
- [x] (2026-08-15 12:39Z) Completed GPT-5.6 Luna max-thinking review and a bounded GPT-5.6 Sol high-thinking control review, then fixed all confirmed findings.
- [x] (2026-08-15 12:43Z) Retained the proposal and this completed plan as branch handoff context, committed the reviewed first implementation, opened the local Playground, and left its server running without pushing.
- [x] (2026-08-15 13:34Z) Replaced the form-heavy interface with the approved compact source bar, exact CLI example buttons, theme-aware transcript terminal, and problem-oriented examples.
- [x] (2026-08-15 13:34Z) Preserved in-tab command navigation while making Clear and Ctrl+K erase only the visible transcript.
- [x] (2026-08-15 13:34Z) Composed the locally built Playground and current documentation into one preview under `/wavepeek/`, with same-origin navigation matching production.
- [x] (2026-08-15 13:45Z) Updated browser/publication checks and durable maintainer documentation, completed bounded Luna review with KISS/YAGNI pass, fixed all six confirmed findings, passed post-review `just ci` and `just check`, committed, and replaced the inspection server without pushing.
- [x] (2026-08-15 14:24Z) Removed the redundant visible page title and replaced it with a compact Releases/copy-to-agent install strip.
- [x] (2026-08-15 14:24Z) Compressed source and command controls, added Help, renamed problem examples, and gave the desktop workspace the remaining viewport without page scroll.
- [x] (2026-08-15 14:24Z) Prepended new transcript entries beneath the fixed command line and synchronized Material palette state across Playground and documentation.
- [x] (2026-08-15 14:29Z) Extended browser checks, completed bounded Luna review with no findings and KISS/YAGNI pass, passed post-review `just ci` and `just check`, committed the implementation, and refreshed the inspection server without benchmark or push.
- [x] (2026-08-15 14:50Z) Put the header tagline inline, hid Playground search, and shortened the copy-to-agent panel so its entire visible prompt wraps without horizontal scrolling.
- [x] (2026-08-15 14:50Z) Combined Commands and waveform source into one toolbar, removed duplicate sidebar shortcuts, and normalized headings, controls, radii, and terminal typography.
- [x] (2026-08-15 14:57Z) Extended browser checks, completed bounded Luna review with KISS/YAGNI pass, fixed both findings, passed post-review `just ci` and `just check`, committed, and refreshed the inspection server without benchmark or push.
- [x] (2026-08-15 15:09Z) Applied the final approved copy for installation, privacy, Surfer, command discovery, and the dash-free inline tagline.
- [x] (2026-08-15 15:17Z) Showed every example query without More/Less, centered the borderless privacy line, placed the installation prompt inline with normal text color, passed focused browser checks plus final `just ci` and `just check`, committed, and refreshed the server without benchmark or push.
- [x] (2026-08-15 15:25Z) Made the inline installation prompt consume all space immediately after its label and replaced separate Run/Stop controls with one stateful button.
- [x] (2026-08-15 15:25Z) Passed focused browser checks, committed, and refreshed the inspection server without benchmark or push.
- [x] (2026-08-15 16:18Z) Moved Output mode beneath Commands with its description inline, reduced the sidebar to Demo queries, and moved transient copy feedback onto the Copy button.
- [x] (2026-08-15 16:18Z) Passed focused browser checks and visually inspected the desktop layout without benchmark or push.
- [x] (2026-08-15 16:26Z) Framed Commands and Output mode as one panel matching Waveform source height, normalized their button styling, grouped commands with separators, and added command-aware selection.
- [x] (2026-08-15 16:26Z) Made command buttons insert command help, reduced Extract AXI to Extract, removed output explanatory copy, and passed focused browser and visual checks without benchmark or push.
- [x] (2026-08-15 16:42Z) Made `help` the selected default command and automatically displayed its successful browser output after the demo source loads; focused browser and visual checks pass.
- [x] (2026-08-15 16:46Z) Reflowed Waveform source into a controls/privacy row and a file/visual-viewer row; focused browser and visual checks pass without benchmark or push.
- [x] (2026-08-15 16:51Z) Normalized the acquisition copy, removed the repeated “get” phrasing, shortened the privacy statement, and passed focused browser checks without benchmark or push.
- [x] (2026-08-15 17:01Z) Removed Ctrl+K behavior/help and replaced the single-line command input with a focused, wrapping, auto-growing overlay that collapses on blur; focused browser and visual checks pass without benchmark or push.
- [x] (2026-08-15 17:05Z) Removed focused-input scrollbars and pinned the prompt and action buttons to the terminal header top while the textarea grows; focused browser and visual checks pass without benchmark or push.
- [x] (2026-08-15 17:14Z) Reordered narrow layouts to Install, Waveform source, Commands/Output mode, then terminal; wrapped command controls, kept source buttons inline, reduced top whitespace, and passed focused desktop/mobile checks without benchmark or push.
- [x] (2026-08-15 17:51Z) Replaced the blue-biased dark styling with centralized graphite surface, control, terminal, text, border, link, focus, success, and error tokens; selected controls now use off-white inverse states.
- [x] (2026-08-15 17:53Z) Stopped Demo queries at content height, added local/demo pressed-state semantics, captured light/dark/focused/narrow screenshots, completed bounded Luna plus ponytail review without unresolved actionable findings, and passed final `just ci` and `just check`.
- [x] (2026-08-15 17:58Z) Aligned slate documentation canvas, text, code, links, and footer with the Playground graphite palette; captured the documentation comparison and added same-origin palette checks.
- [x] (2026-08-15 18:35Z) Added the same responsive installation/copy strip to every generated documentation page, centralized its styling and copy behavior across both builds, completed bounded Luna and ponytail review with no findings, and passed final `just ci` and `just check`.
- [x] (2026-08-15 18:49Z) Matched the documentation strip's x/y position, width, height, and line-height to the Playground strip, including native-scrollbar compensation; focused browser checks pass.

## Surprises & Discoveries

- Observation: The unmodified crate already passes `cargo check --target wasm32-unknown-unknown`.
  Evidence: The target check completed successfully on 2026-08-15; platform work is at I/O boundaries rather than a wholesale engine port.

- Observation: Wellen 0.25.6 supports `simple::read_from_reader` and `stream::read` with `BufRead + Seek`, so both initial parse and FST streaming can use the same immutable in-memory bytes.
  Evidence: `wellen/src/simple.rs:37` and `wellen/src/stream.rs:39` in the Cargo registry.

- Observation: The current Pages workflow deploys a Pages artifact but retains generated HTML in `gh-pages` because Mike needs durable cumulative documentation state.
  Evidence: `tools/docs/publish_docs.py` updates `gh-pages`, verifies the staged bundle, exports its complete tree, and `.github/workflows/docs.yml` passes that export to `actions/deploy-pages`.

- Observation: `mike set-default latest` currently owns root `index.html`, which conflicts with the new current-only root Playground.
  Evidence: `run_mike_deploy()` calls `mike set-default` whenever a release is promoted.

- Observation: The supplied FST parses and returns metadata in ordinary single-threaded browser WASM without cross-origin isolation or WASM threads.
  Evidence: A generated `wasm-bindgen` module run in headless Chrome returned status 0 with `time_unit: 1ps`, `time_start: 1ps`, and `time_end: 1880182ps`.

- Observation: Byte-backed FST streaming also works in browser WASM; no fallback extractor path is needed.
  Evidence: Playwright compared browser and native AXI extraction output byte-for-byte for the supplied demo, including stderr and status.

- Observation: Material's default grid track and content-box buttons allowed mobile children to create horizontal overflow even after the workspace switched to one column.
  Evidence: At 390 px the document initially measured 917 px wide; constraining the Playground grid to `minmax(0, 1fr)` and clipping its outer overflow reduced it to the viewport while retaining scrollable output panes.

- Observation: Async demo and local-file reads need a generation guard because an older read can otherwise replace a newer source choice.
  Evidence: Review of `useDemo()` and the local file handler found both await byte reads before installing the selected source.

- Observation: Deployed versioned-asset absence checks must use the generated WASM subdirectory.
  Evidence: The built artifact is `assets/playground/wasm/wavepeek_bg.wasm`; the control review caught an initial check missing `wasm/`.

- Observation: A terminal with an optional hidden error row cannot rely on implicit placement in fixed CSS Grid rows.
  Evidence: Hiding the empty error moved the transcript into the auto row and stretched the shortcut footer across the remaining height. A column flex layout keeps the command and footer intrinsic while the transcript owns the scrollable remainder.

- Observation: Disabling the Run button does not disable the Enter shortcut, and worker termination alone does not finalize a transcript entry.
  Evidence: Bounded review found that a second Enter could orphan the first running entry and source replacement could terminate its worker without changing its status. `runCommand` now guards the shared running state and source replacement uses the existing stop path.

- Observation: Material scopes palette storage to `config.extra.scope`, which defaulted to each separately built site's current directory.
  Evidence: Browser inspection showed `/wavepeek/.__palette` after toggling the Playground but no shared value in `/wavepeek/latest/`. Setting both generated sites to `/wavepeek/` makes the same Material mechanism persist the palette without custom JavaScript.

- Observation: Changing the copyable prompt from an input to visible rich text invalidated the input-only `.select()` fallback.
  Evidence: Bounded review caught the stale fallback. It now selects the prompt with the standard DOM Selection API, and the browser check forces clipboard rejection to exercise that path.

## Decision Log

- Decision: Publish exactly one replaceable Playground at the site root; accumulate only documentation through Mike.
  Rationale: This is the clarified product requirement and avoids versioned WASM, frontend, and demo copies.
  Date/Author: 2026-08-15 / user and implementation agent.

- Decision: Perform the pre-v3 `gh-pages` cleanup as one reviewed manual branch commit before the first v3 docs publication.
  Rationale: A reusable reset mode is speculative code for a one-time operation. Pages is deployed from an Actions artifact, so the cleanup commit need not become a broken live deployment.
  Date/Author: 2026-08-15 / user and implementation agent.

- Decision: Use the supplied `scr1_axi.fst` as one repository-tracked current demo and retain no historical demo copies.
  Rationale: It is the user-selected artifact, is small enough for browser use, and contains representative AXI activity.
  Date/Author: 2026-08-15 / user.

- Decision: Keep browser execution framework-free: Rust/WASM, a dedicated Web Worker, plain JavaScript, HTML, and CSS.
  Rationale: Native browser APIs cover the UI and worker requirements; a framework or Node toolchain would add ownership without solving a current requirement.
  Date/Author: 2026-08-15 / implementation agent.

- Decision: Use `wasm-bindgen` as the only browser runtime dependency and exchange argv/result data as JSON strings at the binding boundary.
  Rationale: `wasm-bindgen` is required to call Rust from browser JavaScript; JSON uses the already-installed Serde stack and avoids another binding-serialization dependency.
  Date/Author: 2026-08-15 / implementation agent.

- Decision: Disable Material instant navigation rather than add lifecycle code.
  Rationale: Ordinary page navigation remains correct and accessible, while full reloads naturally dispose workers and local state. The proposal explicitly permits this choice.
  Date/Author: 2026-08-15 / implementation agent.

- Decision: Treat FSDB, `skill`, and extraction `--source` as explicit browser-unsupported inputs and hide them from browser help.
  Rationale: The browser has one waveform byte source and no general filesystem. Clear rejection is truthful and avoids a virtual filesystem.
  Date/Author: 2026-08-15 / user and implementation agent.

- Decision: Replace structured command fields with one fixed `$ wavepeek` prompt, an editable argument line, exact CLI command buttons, and a scrollable transcript.
  Rationale: The structured form obscures the CLI and consumes the space needed for readable output. Examples provide discoverability while the argument line remains directly editable.
  Date/Author: 2026-08-15 / user and implementation agent.

- Decision: Clear erases only the visible transcript; command navigation remains in memory until the tab closes or reloads.
  Rationale: This matches terminal clear semantics while preserving the user's short-lived working history without persistence.
  Date/Author: 2026-08-15 / user.

- Decision: Keep separate Playground and Mike documentation builds, but compose them into one local preview and use same-origin `/wavepeek/` links.
  Rationale: Separate builds preserve the current-only app/versioned-doc publication model, while one preview makes local navigation match the deployed Pages artifact.
  Date/Author: 2026-08-15 / user and implementation agent.

- Decision: Use one compact install strip with a Releases link and a copyable natural-language agent prompt instead of OS-specific installation tabs.
  Rationale: The prompt can direct an agent to install the latest release, extract `wavepeek skill`, and follow its harness rules without making the Playground own platform installation logic.
  Date/Author: 2026-08-15 / user and implementation agent.

- Decision: On desktop the page itself stays fixed while the transcript scrolls, and newest transcript entries are prepended directly beneath the command line.
  Rationale: The command prompt is at the top, so reverse-chronological entries keep the latest result spatially adjacent and reclaim the viewport for waveform output.
  Date/Author: 2026-08-15 / user.

- Decision: Commands and source selection share one toolbar, and shortcut help exists only in the terminal footer.
  Rationale: The commands row had enough unused horizontal space for the source controls, while duplicated shortcut documentation consumed sidebar space and created typography inconsistency.
  Date/Author: 2026-08-15 / user.

- Decision: Run and Stop are one stateful button.
  Rationale: A separate disabled Stop control permanently reduced editable command width even though only one action is available at a time.
  Date/Author: 2026-08-15 / user.

- Decision: Output mode belongs beneath Commands, while the sidebar contains only Demo queries.
  Rationale: Output mode directly changes command construction; moving it beside command controls clarifies that relationship and leaves the sidebar focused on query discovery.
  Date/Author: 2026-08-15 / user.

- Decision: Command buttons expose command help and mirror the command currently in the input.
  Rationale: The Demo queries already provide complete runnable recipes; the compact command row is more useful as command-level discovery and orientation.
  Date/Author: 2026-08-15 / user.

- Decision: The Playground opens with `help` selected and its output already rendered.
  Rationale: The first screen explains browser conventions and available commands without requiring an initial action.
  Date/Author: 2026-08-15 / user.

- Decision: Long command input wraps and grows over the transcript only while focused.
  Rationale: Commands remain compact at rest while their full editable argv stays visible without widening or permanently enlarging the terminal header.
  Date/Author: 2026-08-15 / user.

- Decision: Narrow layouts prioritize source selection before command controls and wrap command buttons instead of scrolling them.
  Rationale: This preserves the setup sequence and keeps all controls visible on phone-width screens while leaving the desktop composition unchanged.
  Date/Author: 2026-08-15 / user.

- Decision: The dark Playground and documentation share the same neutral graphite canvas, text, code, and link palette.
  Rationale: Material already shares palette state across both builds; sharing its slate tokens also makes navigation between them visually continuous while Playground-specific control and terminal tokens remain scoped.
  Date/Author: 2026-08-15 / user.

- Decision: One shared static script and stylesheet own the installation strip across Playground and documentation.
  Rationale: The two sites remain separate Material builds, so a small generated DOM component avoids template overrides and keeps copy behavior and responsive styling identical without duplicating them.
  Date/Author: 2026-08-15 / user and implementation agent.

## Outcomes & Retrospective

Milestone 1 is complete. The same Clap parser, engines, renderers, diagnostics, and status mapping now run with explicit argv, writers, and invocation-scoped waveform bytes. Native CLI contract tests, all 670 library tests, strict Clippy, and the WASM target check pass. A scratch headless-Chrome probe successfully ran the real bundled FST through the generated binding.

Milestones 2 and 3 are complete. The current Playground builds as a root Material site with no version selector, and browser checks cover native parity in human/JSON/JSONL modes, AXI streaming extraction, unsupported options, command synchronization, local-file privacy, reload clearing, Stop recovery, Surfer linking, and desktop/mobile layout. Documentation generation now owns Mike versioning separately; publication replaces root app assets only when promoting latest.

Milestone 4 is complete. Maintainer documentation records the browser and publication workflows. Luna review found stale async source selection, incomplete browser help/restrictions, and incomplete deployed historical-asset checks; all were fixed. A bounded Sol control review found the actual `--waves` FSDB path and generated WASM subdirectory cases; both were fixed. KISS/YAGNI passed. Post-review `just ci` and `just check` pass, including 93.08% region, 92.51% function, and 93.66% line coverage, strict docs builds, Playwright parity/privacy checks, and native FSDB checks. The benchmark gate is deferred by maintainer direction until conditions are suitable.

Milestone 5 is complete. The structured form is replaced by a compact source bar, immutable `$ wavepeek` prompt, exact command examples, problem-oriented suggestions, output selector, and one theme-aware scrollable transcript. Clear removes only visible entries; Up and Down retain in-tab command navigation. A bounded Luna review passed KISS/YAGNI and found repeat-Enter, clear-during-run, source-replacement, file-picker keyboard, outcome-label, and standalone docs-serve issues; focused browser checks cover every fix. Post-review `just ci` and `just check` pass with the unchanged 93.08% region, 92.51% function, and 93.66% line coverage minimums.

Milestone 6 is complete. The visible title block is gone; a compact Releases/copy-to-agent strip, compressed source row, Help example, neutral example-query heading, and consistent page gutter leave the desktop viewport to the terminal. Newest transcript entries appear directly below the fixed prompt, desktop page scrolling is replaced by transcript/sidebar scrolling, and responsive layouts retain normal page flow. Native Material `extra.scope` now shares palette state across Playground and documentation. Focused browser checks cover clipboard behavior, palette persistence, no-scroll desktop layout, Help insertion, and newest-first parity. Bounded Luna review reported no findings and passed KISS/YAGNI; post-review `just ci` and `just check` pass.

Milestone 7 is complete. The purpose tagline is inline, Playground search is hidden while documentation search remains, and the two-row install strip shows a short wrapping prompt. Commands and source controls share one toolbar, sidebar shortcut duplication is removed, and one consistent typography/radius scale covers headings, buttons, and terminal text. Bounded Luna review passed KISS/YAGNI and found two issues: the rich-text clipboard fallback and responsive install row count. Both are fixed and covered by browser checks; post-review `just ci` and `just check` pass.

Milestone 8 is complete. Final approved text now drives the accented install prompt, browser-local privacy statement, visual Surfer link, `and more with Help` discovery hint, and dash-free tagline. All eight example queries are permanently visible, with no More/Less state or code. Focused browser checks and final `just ci` and `just check` pass.

Milestone 9 is complete. The dark Playground now uses neutral graphite surfaces, off-white inverse selection, a darker integrated terminal, restrained links and focus, weak panel borders, and semantic status colors. Documentation shares the same slate canvas, text, code, link, footer palette, and responsive installation/copy strip, so same-origin navigation is visually continuous. Demo queries stop at content height. The light theme remains unchanged apart from this shared fit-content correction. Browser checks lock both light reference colors and dark tokens/states across Playground and documentation; desktop, focused-input, long-output, documentation, and narrow screenshots were reviewed directly. Bounded Luna review's test-coverage and one-line ponytail findings are addressed; its suggestion to neutralize the supplied terminal defaults further was rejected because the implemented values are the exact recommended contract values and the resulting screenshots no longer show the prior blue cast.

The composed inspection server remains at `http://127.0.0.1:8000/wavepeek/`, with current source-built documentation at `http://127.0.0.1:8000/wavepeek/latest/`. Host wrapper PID `3097945` owns the serve command and listener PID `3098522`; the rebuilt files are served directly and both URLs return HTTP 200. No benchmark gate or push was performed. The proposal and completed ExecPlan remain under `docs/wip/` as explicit branch handoff context and can be removed before merge after maintainer inspection.

## Context and Orientation

`src/cli/mod.rs` owns Clap parsing, help/version handling, engine dispatch, and process output selection. It currently reads `std::env::args_os()` and sends output through global stdout/stderr. `src/output.rs` already separates rendering from most writing and has a generic `JsonlWriter<W: Write>`, so the minimal change is writer injection rather than new renderers. `src/lib.rs` remains the native process wrapper.

`src/waveform/mod.rs` is the backend-neutral waveform facade. `src/waveform/wellen_backend.rs` currently opens a filesystem path, stores that path for FST streaming rereads, and uses multithreaded FST signal loading. Browser execution needs one invocation-scoped logical filename and immutable bytes. Wellen can parse `Cursor<Arc<[u8]>>`; on WASM the backend must use its single-thread signal loader and recreate a reader over the same bytes for streaming.

All waveform engines call `Waveform::open(path)` directly or through `src/engine/expr_runtime.rs::open_shared_waveform`. To avoid changing every engine signature for a single-worker browser, the browser invocation will install one thread-local, invocation-scoped byte source while the existing command runs, and remove it afterward. `Waveform::open` uses that source only when the requested logical path matches. This is deliberately limited to one source because the product supports one active waveform and no concurrent commands in a worker.

The browser binding will live in `src/browser.rs` behind `cfg(target_arch = "wasm32")`. It accepts a JSON argv array, a logical filename, and waveform bytes, then returns a JSON object containing `stdout`, `stderr`, and integer `status`. JavaScript owns shell-like tokenization because the same token list drives both editable-command synchronization and execution. The tokenizer supports whitespace, single quotes, double quotes, and backslash escapes but performs no shell expansion.

Canonical frontend files live under `web/playground/`. `index.md` provides Material page content, `playground.js` owns controls/history/file selection, `worker.js` owns WASM loading and one-shot calls, and `playground.css` owns the responsive workspace. No generated WASM or MkDocs output is committed. The only committed binary is `web/playground/assets/scr1_axi.fst`.

`mkdocs.yml` is shared Material configuration. `tools/docs/prepare_mkdocs.py` stages the extracted version-matched skill references and generates the documentation config. It will wrap the current docs navigation under a Documentation tab, add an absolute Playground root tab, and enable Mike only in the documentation config. A second function in the same helper stages the current root Playground source, generated WASM glue, and demo into a separate MkDocs config without Mike.

`tools/docs/publish_docs.py` owns release staging. Mike continues adding `<version>/` and copied `latest/` documentation trees to `gh-pages`, but it no longer calls `mike set-default`. When `should_promote_latest()` is true, the helper replaces the owned root Playground files from the current release build; historical repair leaves them byte-for-byte unchanged. Root installers keep their existing promotion behavior. The staged bundle remains the sole Pages artifact source.

`justfile` is the stable local interface. New recipes build and serve the current Playground and run browser checks. Browser tests use Python Playwright with a pinned headless Chromium installed in the shared devcontainer. This is test infrastructure only; the shipped frontend has no Playwright or Node dependency.

## Open Questions

There are no product questions. Implementation discoveries that change a decision must be recorded above before proceeding.

## Plan of Work

### Milestone 1: Prove and expose the shared browser execution path

First add one focused Rust test that opens a tiny VCD and the supplied FST from immutable bytes through the waveform facade. Refactor output writing in `src/output.rs` and CLI execution in `src/cli/mod.rs` so explicit argv and arbitrary `Write` sinks can run the same parser, engine, serializers, diagnostics, and status mapping used by native execution. Keep the existing native wrappers and exact CLI behavior.

Add an invocation-scoped byte source in `src/waveform/mod.rs` and byte-backed construction/streaming in `src/waveform/wellen_backend.rs`. Use single-thread signal loading only on WASM. Reject unmatched logical paths normally. Add `src/browser.rs` and target-specific `wasm-bindgen` configuration in `Cargo.toml`, then prove `info` on FST through generated WASM before continuing.

At the end of this milestone, native tests still pass and a browser-callable function returns the same human/JSON/JSONL text and status as the native binary for representative commands.

### Milestone 2: Build the smallest complete Playground

Add the four framework-free files under `web/playground/` and copy the supplied FST into its assets directory. The page provides bundled/local source selection, status, common command controls, an editable visible command, output mode, Run, Stop, in-memory history, stdout/stderr/status display, and a Surfer link only for the bundled demo.

Use one dedicated worker. Loading a source sends bytes to the worker; every Run invokes fresh Rust parsing and waveform parsing against those bytes. Stop terminates the worker. Reload naturally clears all state. Control edits mutate only their recognized tokens and preserve unknown options; manual edits reparsed into controls never rewrite the command unless the user changes a control. No local source name, bytes, command, or output leaves same-origin requests.

At the end of this milestone, a user can run representative bundled VCD/FST commands, select a local file, stop a run, inspect bounded history, and use the layout on narrow and wide viewports.

### Milestone 3: Integrate current Playground with versioned Material documentation

Remove `navigation.instant` from `mkdocs.yml`, enable top-level tabs, and move Mike configuration into generated documentation config only. Extend `tools/docs/prepare_mkdocs.py` and tests to generate two source/config trees: cumulative versioned documentation and one current root Playground.

Extend `tools/docs/publish_docs.py` and its tests so promoted stable releases replace root Playground output and demo, while old-version repairs preserve them. Remove `mike set-default`; keep `latest` as a copied documentation alias. Extend allowed-path, staged-bundle, root-asset, and exported-artifact checks. Extend `tools/docs/check_deploy.py` to check root Playground markers, reported version, demo bytes/CORS, latest/versioned docs, and absence of Playground payloads under versioned docs.

At the end of this milestone, a local staged Pages tree has `/` as the current Playground, `/latest/` plus `/X.Y.Z/` as documentation, one root demo asset, and no versioned Playground copies.

### Milestone 4: Reproducible browser checks, docs, review, and handoff

Pin `wasm-bindgen-cli`, the WASM target, Playwright, and headless Chromium in the shared devcontainer. Add focused Just recipes and browser integration tests for demo startup, valid/invalid commands, status/channels, command-control synchronization, local-file privacy, reload clearing, Stop recovery, Surfer behavior, navigation, theme, and desktop/mobile layouts. Add native-versus-browser parity fixtures for human, JSON, JSONL, help/version where supported, and failure status.

Update `docs/architecture.md`, `docs/environment.md`, `docs/testing.md`, `docs/quality.md`, `docs/automation.md`, and `tools/docs/README.md` only where durable maintainer behavior changed. Run focused tests after each slice, then `./dev just ci` and `./dev just check`.

Request a read-only multi-focus review using GPT-5.6 Luna with max thinking. Every reviewer prompt must explicitly enforce KISS, YAGNI, the repository's Ponytail policy, and the `ponytail-review` skill's delete-first format in addition to correctness. Apply confirmed findings and rerun affected checks. Spawn GPT-5.6 Sol at high thinking for an independent control opinion if findings are ambiguous or a second review is needed.

Commit coherent slices with conventional messages and active hooks. Do not push. Finally run the local serve recipe in a persistent host-visible process, open the URL with the host browser, verify the process remains alive, and report the URL and server PID.

### Milestone 5: Redesign the terminal workspace and unify local preview

Replace `web/playground/index.md`, `playground.css`, and the UI state in `playground.js` without changing the worker or Rust execution boundary. The source bar stays compact and reports name, size, extension-derived format, and loading state. Exact command buttons and problem-oriented suggestions install proven demo commands but never run automatically. The fixed `$ wavepeek` prefix is not editable. Each run appends command, stdout, highlighted stderr, and a green or red duration badge to one scrollable transcript. Enter runs, Up and Down navigate in-tab commands, and Clear clears only transcript nodes.

Change generated nav links to same-origin `/wavepeek/` paths. Add a Just composition recipe that copies the root Playground build under a preview `wavepeek/` directory and the current generated docs under `wavepeek/latest/`, then serve that static tree. Browser checks use this exact composed layout, click both navigation directions, and validate command insertion, output modes, transcript accumulation, clear/history semantics, source metadata, Stop recovery, local privacy, theme colors, and responsive ordering.

At the end of this milestone, `./dev just playground-test` passes and `./dev just playground-serve` opens `http://127.0.0.1:8000/wavepeek/`; Documentation stays on the same local origin and displays docs built from the current source tree.

### Concrete Steps

Run all Cargo, Python project tooling, MkDocs, browser tests, and quality gates from the repository root through `./dev`.

    ./dev cargo test --lib
    ./dev cargo check --target wasm32-unknown-unknown
    ./dev just playground-build
    ./dev just playground-test
    ./dev just docs-site-check
    ./dev just ci
    ./dev just check

Expected focused browser smoke output includes a successful bundled command:

    status: 0
    stdout: format: FST
    stderr: <empty>

Expected unsupported behavior includes:

    wavepeek extract axi --source config.json --waves scr1_axi.fst
    status: 1
    stderr: fatal: args: --source is not supported in the browser

Before each host commit, ensure the devcontainer is running and use ordinary Git so installed hooks and signing remain active:

    git status --short
    git commit -m "<conventional message>"

At handoff, start without blocking the main shell and open from the host:

    ./dev just playground-serve
    xdg-open http://127.0.0.1:<port>/

The exact persistent launch wrapper and port may vary to keep the process alive after the agent response; record both in `Outcomes & Retrospective`.

### Validation and Acceptance

The implementation is accepted when the current proposal's definition of done is observable and automated. The root page must identify the current version, load `scr1_axi.fst`, run `info`, hierarchy, value/change/property, generic extraction, and AXI examples, and render deterministic channels/status. A user-selected VCD/FST must be processed locally with no data-bearing external request. FSDB, `skill`, and `--source` must be explicit browser errors. Stop must recover by replacing the worker, and reload must return to bundled default state with empty history.

Material navigation must link root Playground to latest docs and every docs version back to root. The version selector must exist only for docs. Repairing a historical docs version must not modify root app/demo blobs. The Surfer URL must contain the current public demo URL and use a safe new tab. Browser tests must cover desktop and narrow viewports and axe-equivalent basic semantics through labels, keyboard-operable native controls, focus visibility, and status live regions.

Native CLI snapshots and behavior must remain unchanged. Browser parity compares output and status with the same latest native source, excluding explicitly unsupported features and platform-specific help text. `just ci` and `just check` must pass after review.

### Idempotence and Recovery

Generated WASM, MkDocs sites, Playwright traces, and logs live only under ignored `tmp/` or `target/`; build recipes remove only their own named outputs before rebuilding. They may be rerun safely. Publication staging continues to use detached/source and `gh-pages` worktrees with existing cleanup helpers and does not push during local checks.

The committed demo copy is never generated from an unverified download: its expected SHA-256 is asserted by tests. If a WASM build fails, remove only its named `tmp/playground-*` output and rerun after rebuilding the devcontainer. If a commit hook fails, fix the cause and commit normally; do not bypass hooks.

The final server PID belongs to this task and may be stopped by the maintainer after inspection. No other process or arbitrary `tmp/` content is removed.

### Artifacts and Notes

The selected demo provenance is:

    source: https://github.com/kleverhq/rtl-artifacts/releases/download/v2.0.0/scr1__max__axi__riscv_compliance.fst
    bytes: 3648879
    sha256: aad73e9b0d2b244b67a96b254371ff29a2ac2e54077176376f6361570789e884

The confirmed Surfer deep-link shape is:

    https://app.surfer-project.org/?load_url=<percent-encoded-public-demo-url>

### Interfaces and Dependencies

In `src/cli/mod.rs`, provide one crate-visible explicit invocation function conceptually equivalent to:

    fn run_from<I, T, O, E>(argv: I, stdout: &mut O, stderr: &mut E, report_machine_errors: bool) -> Result<(), CliFailure>

where argv is explicit and output implements `std::io::Write`. Native `run()` passes process argv and locked process writers. The exact generic spelling may change to satisfy Rust borrowing without adding a trait layer.

In `src/waveform/mod.rs`, provide one invocation-scoped function conceptually equivalent to:

    fn with_waveform_bytes<T>(name: PathBuf, bytes: Arc<[u8]>, run: impl FnOnce() -> T) -> T

`Waveform::open(path)` consumes the byte source only when `path == name`; otherwise native path behavior remains unchanged. A comment records the one-worker concurrency ceiling and the explicit-input upgrade path if concurrent browser invocations become a real requirement.

In `src/browser.rs`, export one `wasm_bindgen` function that accepts argv JSON, logical filename, and `&[u8]`, and returns serialized:

    {"stdout":"...","stderr":"...","status":0}

Use target-specific `wasm-bindgen = "=0.2.127"`. Do not add a JavaScript framework or serialization crate.

The browser test dependency is Python `playwright==1.62.0` with its Chromium headless shell. It is development infrastructure only and must not appear in runtime frontend assets.

The publication helper owns these generated roots under its existing work directory: one extracted skill tree, one versioned-doc MkDocs tree/config/site, and one current-Playground MkDocs tree/config/site. Generated names must be explicit in `Paths`; do not introduce a generic artifact registry or plugin abstraction.

Revision note (2026-08-15): Reopened the completed plan for the user-approved terminal redesign and a same-origin composed local preview. The Rust/WASM engine and production publication model remain unchanged.
