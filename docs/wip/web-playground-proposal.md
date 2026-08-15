# WavePeek Browser Playground Proposal

## Status

Proposed for WavePeek v3.

WavePeek v3 is a deliberate public compatibility reset. This proposal does not preserve, migrate, or emulate the v2 documentation site or any earlier public documentation layout. Pre-v3 documentation may remain discoverable through repository history, tags, and releases, but it must not remain published on the WavePeek GitHub Pages site and must not constrain the v3 implementation.

## Objective

Add a browser-based WavePeek Playground to the existing Material for MkDocs website.

The Playground must let a user run real WavePeek CLI commands against either:

- a bundled public demo waveform; or
- a local VCD/FST selected from the user's machine.

All waveform processing must happen in the user's browser. A local waveform must never be uploaded to WavePeek infrastructure or any third party.

The final result should feel like using the actual WavePeek CLI without installing it, while remaining an integrated part of the documentation site rather than a separate product.

## Product Vision

The public WavePeek site root is one interactive Playground built from the latest stable WavePeek release. It is replaced when a newer release is published and is not retained per documentation version. The same Material for MkDocs shell provides access to both the current Playground and the versioned documentation.

A typical user flow is:

1. Open the WavePeek site and land on the Playground.
2. Use the bundled demo waveform immediately, without selecting a file.
3. Choose an example command or edit the command line directly.
4. Run one WavePeek invocation and inspect its stdout, stderr, and status.
5. Repeat with other commands.
6. Optionally select a local VCD/FST and run the same CLI against it.
7. For the bundled demo, open the exact same public waveform visually in the Surfer web application.
8. Move between Playground and documentation through the normal Material navigation.

The Playground is a CLI demonstration and inspection environment. It is not a replacement for the native CLI for large production waveforms.

## Core Boundaries

### The Playground is stateless at the WavePeek execution level

Every press of `Run` or equivalent action represents one independent WavePeek invocation.

For each invocation, the implementation must:

1. parse the currently visible command;
2. create fresh command execution state;
3. open and parse the selected waveform for that invocation;
4. execute exactly one WavePeek command;
5. capture its stdout, stderr, and exit status;
6. discard all parsed waveform and invocation state after completion.

The browser may retain the immutable source bytes of the active waveform for the lifetime of the page so it does not need to reread the browser `File` object or refetch the bundled asset before every command. It must not retain or reuse a parsed WavePeek waveform, hierarchy, signal cache, command engine, or equivalent state between invocations.

This preserves the core WavePeek model: one process-like invocation, one command, one result.

### The Playground is not a REPL or shell

The interface may look like a terminal and may show a history of completed commands, but it must not claim or implement a shell, PTY, or persistent WavePeek session.

The v3 scope does not include:

- pipes;
- redirects;
- shell expansion;
- environment-variable expansion;
- a current working directory;
- `cd` or general filesystem commands;
- background processes;
- command composition with shell operators;
- persistent variables;
- reuse of prior command results as hidden input;
- a long-lived parsed waveform session.

A command entered into the runner is one WavePeek command line, not a shell program.

### The visible command line is the source of truth

The complete command that will be executed must always be visible and editable before execution.

The Playground may expose graphical controls for common operations, including:

- selecting the bundled demo or a local waveform;
- selecting an example command;
- choosing Human, JSON, or JSONL output where supported;
- setting common command options;
- filling frequently used scopes, signals, times, expressions, or extractor parameters.

These controls must edit the visible command line. They must not add hidden arguments, maintain a second authoritative command model, or execute through a separate web-only API whose behavior differs from the displayed command.

The command line remains freely editable. The user may alter or delete any generated token, paste a command from the documentation, or use an option for which no graphical control exists.

### Bidirectional command/control synchronization

The graphical controls and command line must remain synchronized with low-latency feedback:

- a control change updates the visible command line before execution;
- a manual command edit updates every control that can be derived unambiguously from the command;
- arguments not understood by the graphical controls remain untouched;
- an incomplete, invalid, or custom command remains editable and runnable;
- controls may enter a neutral or custom state when the command cannot be represented exactly;
- the Playground must not silently rewrite a manually edited command merely to make it fit the controls;
- clicking a control should make the smallest practical deterministic edit while preserving unrelated user arguments.

There is no required `Guided` versus `Manual` mode toggle. A single editable command line with bidirectional controls is the desired product model. The implementation may choose its internal parsing and reconciliation strategy as long as the behavior above is preserved.

## Browser CLI Contract

### Command syntax

The Playground must accept the normal WavePeek v3 command syntax, including the leading executable name, for example:

```text
wavepeek scope --waves scr1_axi.fst --tree
```

Waveform examples must include a visible `--waves <FILE>` argument. Selecting a waveform in the GUI should update or add this argument in the visible command where appropriate. The argument remains manually editable.

The implementation may tokenize the command line without implementing a full shell. It must support the quoting needed by valid WavePeek commands and expressions.

### Supported surface

The browser build must support the WavePeek v3 CLI surface that is meaningful in a browser, including:

- VCD input;
- FST input;
- waveform metadata and hierarchy commands;
- signal discovery and sampling commands;
- change and property commands;
- all v3 extraction commands that otherwise support VCD/FST;
- Human output;
- JSON output where the native command supports it;
- JSONL output where the native command supports it;
- root help and command help;
- version output.

WavePeek v3 no longer has the old `docs` or `schema` commands. The Playground must not recreate them.

### Explicitly unsupported surface

The following are outside the browser-supported surface:

- FSDB input;
- the `skill` command;
- every extraction command's `--source <FILE>` configuration-file option.

Attempting to use FSDB, invoke `skill`, or pass an extraction `--source` option must produce a clear, user-facing unsupported-in-the-browser result rather than a crash, hang, or misleading file error. The browser does not provide a second JSON-file upload or a general virtual filesystem. The exact internal mechanism is implementation-defined, but help and runtime behavior must not falsely imply that these features work in the Playground.

### Native parity

The Playground must reuse the WavePeek CLI parser, command semantics, engine behavior, diagnostics, and renderers wherever technically possible. It must not independently reimplement the meaning of commands in JavaScript.

For the same WavePeek version, logical input filename, waveform bytes, and arguments, browser and native runs over the supported surface should produce equivalent:

- stdout;
- stderr;
- exit status;
- Human formatting;
- JSON documents;
- JSONL records;
- diagnostics and ordering.

Platform-specific differences must be minimized and documented if unavoidable. Parity should be enforced by automated tests over representative fixtures and commands, not assumed from architecture alone.

## Terminal-Style Runner Experience

The Playground should present a terminal-style command runner with:

- one editable command input;
- a clear `Run` action and keyboard shortcut;
- a visible running state;
- a history of completed one-shot invocations during the current page lifetime;
- distinct presentation of the entered command, stdout, stderr, exit status, and optionally elapsed time;
- example controls that populate the command without immediately executing it;
- copy-friendly output.

The exact visual design is not prescribed. It should be consistent with the Material site, work in both light and dark themes, and remain usable on typical desktop and mobile layouts.

Only one active invocation is required for the initial implementation. Parallel execution, job management, and persistent queues are not required.

## Waveform Sources

### Bundled default demo

WavePeek v3 must ship with one bundled default demo waveform that is available immediately when the Playground opens.

The initial demo is `scr1_axi.fst`, sourced from:

```text
https://github.com/kleverhq/rtl-artifacts/releases/download/v2.0.0/scr1__max__axi__riscv_compliance.fst
```

Its SHA-256 digest is `aad73e9b0d2b244b67a96b254371ff29a2ac2e54077176376f6361570789e884`. A repository-tracked copy, not the external release URL, is the publication source.

The demo must support the main WavePeek workflows, including at least:

- `info` or equivalent metadata inspection;
- hierarchy exploration;
- signal discovery;
- point value inspection;
- change inspection;
- property evaluation;
- generic extraction where applicable;
- AXI extraction.

The initial release does not require a demo catalog, multiple selectable bundled demos, remote release-asset discovery, or integration with another repository's asset releases. Those may be added later without changing the core Playground contract.

### Single current demo asset

The bundled waveform must live outside WavePeek's versioned documentation directories so it is not copied once per Mike version. Its public path is conceptually:

```text
/assets/playground/scr1_axi.fst
```

The exact path is implementation-defined, but the following rules are mandatory:

- the asset is stored in the WavePeek repository;
- the publishing pipeline deploys it to a shared non-versioned GitHub Pages path;
- only the demo used by the current Playground needs to remain published;
- replacing the demo must be part of the same deployment as the matching Playground;
- superseded demo assets need not be retained;
- the asset is not duplicated under any versioned documentation tree.

A future external asset source may be introduced separately. The Playground must remain self-contained and must not depend on another repository or service for its default demo.

### Local user waveform

The user must be able to select one local VCD or FST through browser file-selection facilities.

The local file's basename becomes its logical filename in the Playground and is inserted into the visible `--waves` argument when selected. The user may edit that argument afterward. Invalid or unavailable logical names should fail through normal command error handling.

The initial scope requires only one active user-selected file at a time. Multiple uploaded files, directories, renaming, and a general virtual filesystem are not required.

The user must be able to return to the bundled demo after selecting a local file.

## Privacy and Ephemeral State

All processing of a local waveform must occur in the browser.

The Playground must not transmit any of the following to WavePeek infrastructure, analytics services, error-reporting services, or other third parties:

- local waveform bytes;
- local filenames;
- entered commands;
- stdout or stderr;
- parsed waveform information;
- command history.

The Playground must not persist its user data in IndexedDB, local storage, service-worker caches intended for user state, or any server-side store.

Reloading or closing the page must clear:

- the local waveform;
- command and invocation history;
- command output;
- Playground-specific selections and errors derived from the local file.

Material's normal theme preference or other generic documentation-site preferences may continue to behave as they already do, provided they do not contain Playground user data.

The implementation should include an automated browser-level network test demonstrating that selecting and processing a local waveform does not result in an upload or data-bearing external request.

## Performance and Resource Behavior

The Playground operates best-effort within browser and WebAssembly memory limits.

The product contract must not impose an arbitrary file-size limit solely for convenience. The UI may warn that large waveforms can be slow or exceed browser memory and should recommend the native CLI for serious or very large workloads.

Parsing and command execution must not leave the page permanently unresponsive. Running the WavePeek workload in a dedicated worker is a reasonable default, but the exact concurrency mechanism is not prescribed.

Out-of-memory conditions, unsupported encodings, malformed files, parser failures, and other errors must surface as comprehensible results. They must not silently upload the file, fall back to a server, or leave stale successful output presented as the latest result.

## Material for MkDocs Integration

The Playground must be part of the existing Material for MkDocs site, not a separately navigated website.

It must reuse the site's normal:

- header and branding;
- light/dark theme behavior;
- GitHub link and other global actions;
- responsive layout conventions;
- top-level navigation.

The desired top-level information architecture is:

```text
Playground | Documentation
```

The site must behave as follows:

- the public site root opens the current Playground;
- the Documentation tab from the Playground enters the latest documentation;
- Mike versioning and the version selector apply only to documentation pages;
- selecting an older documentation version does not select an older Playground;
- the Playground link from every documentation version opens the same current Playground;
- historical documentation directories do not contain Playground builds or frontend assets.

The exact documentation URL prefix and template implementation are intentionally not fixed. The implementation may use a Material custom page template, generated page, plugin, or another maintainable mechanism. It must preserve one coherent Material site and must not use an iframe to embed the WavePeek Playground.

The Playground may contain a small frontend application, but it must not take over documentation routing or duplicate the site's header or navigation system. The documentation version selector must not be shown as if it controls the Playground.

If Material instant navigation remains enabled, repeated navigation between documentation and Playground must initialize and dispose the Playground correctly. It must not create duplicate event handlers, multiple workers, stale waveform state, or a requirement for a full browser refresh.

## Versioning and Compatibility Contract

### v3 public reset

WavePeek v3 is the first version of the new public site and Playground contract.

The implementation must:

- remove v2 and earlier documentation from the published GitHub Pages site;
- remove those versions from the public version selector;
- avoid legacy redirects, compatibility pages, dual site layouts, migration code, or fallback behavior for pre-v3 docs;
- avoid designing the v3 site around preservation of the old publication format.

The one-time reset may be performed by an intentional reviewed commit to the `gh-pages` state branch immediately before the first v3 documentation publication. No reusable reset mode is required in the publication code. Because the live site is deployed from a Pages artifact, the reset commit itself must not be deployed separately.

Historical documentation remains available only through repository history, tags, and release source where applicable.

### Current-only Playground

Exactly one Playground is published at the non-versioned site root. It must be built from the latest stable WavePeek release and replaced when a newer stable release is published.

Documentation remains cumulative and versioned through Mike, but Playground builds do not. Switching the documentation version must not change:

- the Playground implementation;
- the WavePeek command behavior available in the Playground;
- the Playground help and examples;
- the bundled demo waveform.

The Playground must report the current stable WavePeek version. Any native installation recommendation shown by the Playground must point to the latest stable native release rather than the documentation version from which the user navigated. Publishing or repairing an older documentation version must leave the current Playground and demo unchanged.

### Publishing

The existing release-driven documentation and GitHub Pages pipeline should remain the single publication path. The `gh-pages` branch remains cumulative state for versioned documentation, while the root Playground and its assets are replaceable current state.

A release publication should conceptually:

1. build the browser artifact from the exact stable release source when that release becomes the latest stable version;
2. build and add that release's versioned MkDocs documentation through Mike;
3. replace the non-versioned root Playground and frontend assets;
4. publish the one current bundled demo outside the documentation trees;
5. remove superseded Playground-only assets that are no longer referenced;
6. export and deploy one coherent GitHub Pages artifact;
7. verify the current Playground, versioned documentation navigation, waveform URL, and Surfer integration.

The exact workflow decomposition is implementation-defined.

## Surfer Integration

The Playground must explain that Surfer provides visual waveform exploration while WavePeek provides deterministic CLI inspection.

### Bundled demo

When the bundled demo is active, the Playground must offer an action equivalent to:

```text
Open this waveform visually in Surfer
```

The action must:

- open the Surfer web application in a new tab;
- pass the public URL of the exact same current demo waveform used by the Playground;
- cause Surfer to load that waveform automatically;
- use safe external-link behavior.

The deployment pipeline should verify that the demo asset is publicly fetchable with the cross-origin behavior required by Surfer.

### Local waveform

A local user waveform must never be uploaded or converted into a public URL merely to support Surfer.

When a local file is active:

- automatic open-in-Surfer behavior must be disabled or visibly unavailable;
- the UI must explain that automatic loading is available only for the bundled public demo;
- the user should still be able to open the Surfer website and be instructed to select the same local file there manually.

The UI must not imply that the local file has been transferred to Surfer.

## Error and Diagnostic Behavior

The Playground must preserve normal WavePeek errors wherever the command can reach the native command path.

Browser-specific boundary errors must be explicit and actionable, including at least:

- local filename not available in the Playground;
- unsupported FSDB input;
- unavailable `skill` command;
- malformed command-line quoting;
- browser memory exhaustion or allocation failure where detectable;
- failure to fetch the bundled demo;
- unsupported browser capability needed to run the build.

Browser-specific errors should remain separate from successful stdout and should not masquerade as valid WavePeek output.

## Required Tests

The implementation must include automated coverage for the product boundaries, not only low-level WASM compilation.

### Native/browser parity

Use representative VCD and FST fixtures to compare native and browser results for supported commands, including:

- help and version;
- metadata;
- hierarchy and signal listing;
- value sampling;
- changes;
- properties;
- generic extraction;
- the protocol extractor demonstrated by the bundled waveform;
- Human, JSON, and JSONL modes where applicable;
- success and failure exit paths.

### Stateless invocation behavior

Tests must demonstrate that separate runs do not share parsed waveform or command-engine state and that changing the active source produces a fresh invocation.

### Command/control synchronization

Tests must cover:

- controls updating the visible command;
- manual command edits updating recognizable controls;
- preservation of unknown/custom arguments;
- incomplete or invalid commands remaining editable;
- output-mode controls adding and removing visible flags without hidden behavior;
- waveform selection updating the visible logical filename;
- running exactly the currently displayed command.

### Browser integration

End-to-end tests should cover:

- first load with the bundled demo;
- executing a representative demo command;
- selecting and executing against a local VCD/FST;
- no local-file upload or data-bearing external request;
- reload clearing all Playground user state;
- navigation from Playground to latest documentation and from every documentation version back to the current Playground, including instant navigation if enabled;
- documentation version switching without switching or duplicating the Playground;
- absence of the documentation version selector from the Playground experience;
- light and dark theme behavior;
- clear FSDB, `skill`, and extraction `--source` unsupported results;
- active Surfer deep link for the bundled demo;
- unavailable automatic Surfer loading for a local waveform.

### Deployment verification

The release workflow should verify:

- the non-versioned site root opens the current Playground;
- latest and historical v3-or-later documentation is reachable through the shared Material navigation;
- documentation version switching does not switch the Playground;
- the browser artifact reports the latest stable WavePeek version;
- pre-v3 documentation is absent from GitHub Pages;
- the current demo asset exists at one non-versioned URL;
- the Playground and demo are not duplicated into versioned documentation trees;
- Surfer can fetch the demo URL;
- the deployed page can execute at least one smoke command.

## Non-Goals for the Initial Feature

The following are explicitly outside this proposal:

- preserving or migrating the WavePeek v2 documentation site;
- a legacy documentation archive on GitHub Pages;
- a stateful WavePeek session;
- caching parsed waveforms between commands;
- a REPL, shell, PTY, pipes, or redirects;
- an embedded visual waveform viewer;
- embedding Surfer in an iframe;
- automatically transferring a local waveform to Surfer;
- FSDB support in WebAssembly;
- browser execution of `wavepeek skill`;
- extraction configuration through `--source <FILE>`;
- reintroducing removed `docs` or `schema` commands;
- versioned or archived Playground deployments;
- historical copies of bundled demo assets;
- multiple simultaneous user uploads;
- a general browser filesystem;
- a bundled demo catalog;
- automatic discovery of waveform assets from another repository or release service;
- persistent command history or local waveform restoration;
- server-side execution or upload fallback;
- telemetry over filenames, commands, outputs, or waveform contents;
- guaranteed support for arbitrarily large production waveforms.

## Implementation Freedom

This proposal defines the external behavior and boundaries, not a mandatory internal design.

The implementation may choose, among other things:

- the Rust/WASM binding mechanism;
- the internal path-versus-bytes source abstraction;
- worker and message-passing design;
- vanilla JavaScript, TypeScript, or a small frontend framework;
- the Material extension mechanism;
- the command-line reconciliation algorithm;
- exact layout and component styling;
- internal packaging of browser-specific code;
- how stdout, stderr, and exit status are represented across the WASM boundary;
- how release artifacts are staged before the final Pages deployment.

These choices are acceptable only if they preserve the stated CLI parity, stateless execution, privacy, visible-command, current-only Playground, versioned-documentation, MkDocs integration, and shared-asset contracts.

## Definition of Done

The feature is complete when all of the following are true:

1. WavePeek v3 is the new public site baseline, and pre-v3 documentation is no longer available on GitHub Pages.
2. Opening the site root lands on the one current Playground built from the latest stable WavePeek release.
3. Mike accumulates only v3-or-later documentation; the documentation version selector does not switch or duplicate the Playground.
4. The bundled `scr1_axi.fst` demo loads without user setup and supports representative commands, including AXI extraction.
5. The current demo waveform is published at one shared non-versioned URL and is not duplicated under any documentation version.
6. A user can select a local VCD/FST and process it entirely in the browser.
7. Each run is a fresh one-shot WavePeek invocation with no parsed-state reuse.
8. The visible editable command is exactly the command executed.
9. GUI controls and manual command edits synchronize bidirectionally without hiding arguments or destroying custom input.
10. Supported browser commands exhibit tested parity with the latest stable native WavePeek version.
11. FSDB, `skill`, and extraction `--source` options fail clearly as browser-unsupported features.
12. Reloading the page removes the local file, history, outputs, and Playground user state.
13. No local waveform data, filename, command, or output is uploaded or sent to telemetry.
14. The bundled demo can be opened automatically in Surfer from its public URL.
15. A local waveform is never transferred automatically to Surfer; the user is directed to load it there manually.
16. Navigation, theming, documentation version switching, and repeated entry to the current Playground work as part of Material for MkDocs.
17. The release pipeline builds, publishes, and smoke-tests one coherent site containing the current Playground and cumulative v3-or-later documentation.
