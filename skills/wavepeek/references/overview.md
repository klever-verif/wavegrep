# Command overview

Use this topic to choose the right command family. It is not the exact flag reference. For exact syntax, defaults, required flags, and examples, run `wavepeek help <command-path...>` or `wavepeek <command> --help`.

## Help and documentation

Use `help` when you want exact syntax, especially for nested paths such as `wavepeek help extract axi`. Use the relative links in this package for narrative guidance, workflows, troubleshooting, and stable semantics.

## Helper commands

Read [Machine output](machine-output.md) when a client needs JSON envelope or JSONL stream details.

Use `wavepeek skill <DIRECTORY>` to extract the complete, version-matched package into a new or empty directory.

## Waveform inspection commands

Use `info` first when you need dump metadata before running time-based queries. It reports the dump time unit and normalized start/end bounds that other commands use.

Use `scope` to explore hierarchy structure. It is the stable way to discover scope paths before narrowing later queries to a smaller part of the design.

Use `signal` after `scope` to inspect the signals available in a selected scope. Recursive mode broadens that view into child scopes while preserving deterministic ordering.

Use `value` for exact point sampling. It is the most direct command when you already know the signal set and want one or more explicit normalized timestamps.

Use `change` for event-aligned signal tables or sparse value transitions across a bounded time range. Trigger selection comes from `--on`; `--row-mode` selects dense or sparse rows, and `--row-values` selects full or delta values. Expression syntax is documented in [Expression language](expression-language.md).

Use `property` when you want to evaluate a logical expression on event-selected timestamps instead of printing raw signal snapshots. Capture modes control whether you keep every match or only state transitions such as asserts and deasserts.

Use `extract` commands when you want one row per matching synchronous event with ordered payload values sampled at the pre-edge point. `extract ahb` tracks manager-facing AHB-Lite or AHB5 address/data pipeline events, including delayed completions and synchronization boundaries. `extract apb` covers APB3, APB4, and APB5 Setup, waited Access, and completed Access states from Arm IHI 0024E Issue E. `extract atb` covers AMBA ATB transfer, flush, and synchronization-request events. `extract axi` covers AXI3, AXI4, AXI4-Lite, AXI5, AXI5-Lite, ACE, ACE-Lite, ACE5, ACE5-Lite, ACE5-LiteDVM, and ACE5-LiteACP ready/valid channel transfers. `extract axistream` covers one AXI4-Stream or AXI5-Stream interface, including an explicit mode for physically omitted `TREADY`. `extract generic` covers custom handshakes, FIFO pushes and pops, and other transfer-like rows.

When choosing between VCD, FST, and FSDB input or diagnosing unexpectedly slow queries, use [Waveform performance](waveform-performance.md) for format-level performance guidance.

## Which document is normative?

Use this overview to choose a command quickly. When exact flags matter, defer to generated help. For behavioral semantics, use the [Command model](command-model.md). For exact JSON and JSONL shapes, use [Machine output](machine-output.md).
