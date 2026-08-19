# Explore a dump

Start here when you have a waveform file but do not yet know its time range, hierarchy, or signal names. The usual flow is `info`, then `scope`, then `signal`.

## How long is this waveform?

Check the dump bounds before choosing timestamps or time windows:

```text
$ wavepeek info --waves dump.fst
time_unit: 1ns
time_start: 0ns
time_end: 250000ns
```

The dump may use a different unit or start after zero. Reuse the reported units in later `--at`, `--from`, and `--to` arguments.

## Which blocks were dumped?

A shallow tree is usually enough to find the design root:

```text
$ wavepeek scope --waves dump.fst --tree --max-depth 2 --max 30
tb kind=module
└── dut kind=module
    ├── cpu kind=module
    ├── dma kind=module
    └── peripherals kind=module
```

If the hierarchy is large, filter by a likely block name:

```text
wavepeek scope --waves dump.fst --filter '.*(cpu|axi|uart).*' --max-depth 4
```

The filter matches full canonical scope paths.

## What is the exact signal name?

Once you know the scope, search its direct signals:

```text
$ wavepeek signal --waves dump.fst --scope tb.dut.cpu --filter '.*(clk|reset|state).*'
clk kind=wire width=1
reset_n kind=wire width=1
state kind=reg width=4
```

Add `--recursive` when the signal might be in a child scope. Keep the search shallow at first:

```text
wavepeek signal --waves dump.fst \
  --scope tb.dut \
  --recursive --max-depth 2 \
  --filter '.*(valid|ready|data).*' \
  --abs --max 30
```

`--abs` prints canonical paths that can be copied into later commands. Without it, recursive output uses names relative to the selected scope.

## What if the result is truncated?

`scope` and `signal` limit their output by default. A truncation warning means the query found more matches than it printed. Narrow the filter or hierarchy depth before raising `--max`; this usually produces a more useful result than dumping the entire design.

See [Paths, signals and scopes](paths.md) for name resolution and [Time units and windows](timeunits.md) for time-token rules.
