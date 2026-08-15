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
- [x] (2026-08-15 12:43Z) Retained the proposal and this completed plan as branch handoff context, committed the final reviewed code, opened the local Playground, and left its server running without pushing.

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

## Outcomes & Retrospective

Milestone 1 is complete. The same Clap parser, engines, renderers, diagnostics, and status mapping now run with explicit argv, writers, and invocation-scoped waveform bytes. Native CLI contract tests, all 670 library tests, strict Clippy, and the WASM target check pass. A scratch headless-Chrome probe successfully ran the real bundled FST through the generated binding.

Milestones 2 and 3 are complete. The current Playground builds as a root Material site with no version selector, and browser checks cover native parity in human/JSON/JSONL modes, AXI streaming extraction, unsupported options, command synchronization, local-file privacy, reload clearing, Stop recovery, Surfer linking, and desktop/mobile layout. Documentation generation now owns Mike versioning separately; publication replaces root app assets only when promoting latest.

Milestone 4 is complete. Maintainer documentation records the browser and publication workflows. Luna review found stale async source selection, incomplete browser help/restrictions, and incomplete deployed historical-asset checks; all were fixed. A bounded Sol control review found the actual `--waves` FSDB path and generated WASM subdirectory cases; both were fixed. KISS/YAGNI passed. Post-review `just ci` and `just check` pass, including 93.08% region, 92.51% function, and 93.66% line coverage, strict docs builds, Playwright parity/privacy checks, and native FSDB checks. The benchmark gate is deferred by maintainer direction until conditions are suitable.

The inspection server is running at `http://127.0.0.1:8000/wavepeek/`. Host wrapper PID `2973169` owns the devcontainer serve command and listener PID `2973720`; `xdg-open` was invoked and an HTTP request followed the root redirect to a 200 response. No push was performed. The proposal and completed ExecPlan remain under `docs/wip/` as explicit branch handoff context and can be removed before merge after maintainer inspection.

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
