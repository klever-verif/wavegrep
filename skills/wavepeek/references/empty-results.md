# Empty results

An empty result is not always a failure.

In `wavepeek`, many queries are allowed to succeed even when nothing matched. The fix is usually to widen or correct the query, not to debug the CLI itself.

## Know the difference between empty success and real failure

A real failure prints `fatal: ...` on stderr and exits non-zero.

An empty-but-valid query stays successful and usually prints no data rows. List and search-style waveform commands also emit a `WPK-W0003` diagnostic:

- `scope`: no matching scopes printed,
- `signal`: no matching signals printed,
- `property`: no captured events printed,
- `change`: no rows printed.

## `scope` and `signal` can return empty matches

Hierarchy and signal discovery commands also allow empty success.

Common causes:

- the regex is too narrow,
- the expected block or signal is spelled differently in the dump,
- the search is happening at the wrong scope,
- `signal --filter` is matching the leaf signal name, not the displayed recursive prefix,
- `scope --filter` is matching the full canonical scope path.

When in doubt, remove the filter first and confirm the raw names that the dump actually contains.

## `change` can be valid but still show no rows

`change` is the easiest command to misread here.

A dense query can be empty when no event was selected. A sparse query can be fully valid, select events, and still show no rows if none of the requested `--signals` changed at those sampled timestamps.

Common causes:

- the selected time window is too narrow,
- the trigger never fires in that window,
- sparse mode selected events, but the printed signals did not change,
- sparse mode sampled a correct signal list whose values were already stable,
- a signal spelling does not resolve inside the selected scope.

If `change` finds no qualifying rows, it emits a mode-specific diagnostic instead of failing:

```text
warning[WPK-W0003]: no selected events found in selected time range
```

With `--row-mode sparse`, the message is `no signal changes found in selected time range`.

## `property` can succeed with no output

`property` returns rows only when the selected timestamps satisfy the chosen capture mode. If no row matches, it emits `WPK-W0003` and still exits successfully.

That means empty output is normal when:

- the trigger selected no timestamps,
- the predicate never became true,
- `--capture assert` or `--capture deassert` asked for a transition that never happened,
- the window starts after the transition you were hoping to see.

## Most empty results come from one of four mistakes

1. **Wrong names** — the scope path or signal spelling is wrong.
2. **Wrong naming context** — a short name was used without `--scope`, or a canonical path lies outside the selected scope.
3. **Wrong filter or trigger** — the regex or event expression is narrower than intended.
4. **Wrong time window** — the interesting activity is outside the selected bounds.

## Recover safely

Use this order so each retry stays explainable:

1. Run `wavepeek info` to confirm dump bounds and time unit.
2. Run `wavepeek scope` and `wavepeek signal` to confirm the exact names.
3. Remove filters or simplify the trigger.
4. Widen the time window.
5. Add constraints back one at a time until the empty result reappears.

That sequence usually reveals whether the real issue is naming, filtering, or timing.
