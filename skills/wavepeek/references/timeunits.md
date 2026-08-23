# Time units and windows

This page describes the time tokens that `wavepeek` accepts as input and produces in output.

## Dump time

Use `wavepeek info` first. It tells you the dump bounds and the dump time unit that later commands validate against.

## Time tokens

Commands that accept point sampling `--at` (`value`) or window boundaries `--from`/`--to` (`change`, `property`, `extract`) consume input time tokens as arguments.

Every explicit input time token requires an integer magnitude plus a unit suffix. The accepted suffixes are `zs`, `as`, `fs`, `ps`, `ns`, `us`, `ms`, and `s`. Bare numbers or floats are invalid. When `wavepeek` parses a time token, it converts that value into the dump's native `time_unit`. All observable timestamps within output are rendered as normalized integer counts in that dump unit.

Valid tokens:

```text
10ps
25ns
1us
```

## Conversion helpers

The bundled scripts avoid manual unit conversion and floating-point rounding. Run them from the extracted skill directory.

Convert a value to the largest exact integer unit accepted by `wavepeek`:

```text
$ python3 scripts/time_convert.py 1.5ns
1500ps
```

Select an output unit when a decimal result is useful for display:

```text
$ python3 scripts/time_convert.py 1500ps --to ns
1.5ns
```

Add or subtract values with different units:

```text
$ python3 scripts/time_math.py 10ns + 250ps
10250ps
```

An explicit `--to` conversion can produce a decimal value, which `wavepeek` does not accept as a time token. Omit `--to` when the result will be passed to a `wavepeek` command.

## Time windows

Commands that accept `--from` and `--to` (`change`, `property`, `extract`) interpret them as an inclusive time window.

- `--from` plus `--to` means the closed interval from the start token through the end token.
- `--from` without `--to` means from that timestamp through the end of the dump.
- `--to` without `--from` means from the start of the dump through that timestamp.
- Omitting both means the entire dump.

## Troubleshooting

Most time-related failures come from one of four causes:

- the token has no unit,
- the token is not an integer,
- the token is outside the dump bounds,
- the token cannot be represented exactly at the dump resolution.

Fix the token or check the dump attributes with `wavepeek info`.
