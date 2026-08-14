# Scoped vs canonical names

Without `--scope`, signal references are canonical paths from the dump root, such as `top.cpu.clk`.

With `--scope top.cpu`, references may use either:

- a path relative to that scope, such as `clk`, or
- a canonical path inside that scope, such as `top.cpu.clk`.

Both forms resolve to the same waveform signal and may be mixed in one request.

## Without `--scope`

Use full canonical paths:

```text
wavepeek value --waves dump.vcd --at 10ns --signals top.cpu.clk,top.cpu.state
wavepeek change --waves dump.vcd --signals top.cpu.clk,top.cpu.state --on 'posedge top.cpu.clk' --from 0ns --to 20ns
wavepeek property --waves dump.vcd --on 'posedge top.cpu.clk' --eval "top.cpu.state == 8'h03"
```

A short name such as `clk` resolves only if the dump contains that exact top-level canonical path.

## With `--scope <path>`

Use whichever form is convenient for each reference:

```text
wavepeek value --waves dump.vcd --at 10ns --scope top.cpu --signals clk,top.cpu.state
wavepeek change --waves dump.vcd --scope top.cpu --signals clk,top.cpu.state --on 'posedge top.cpu.clk' --from 0ns --to 20ns
wavepeek property --waves dump.vcd --scope top.cpu --on 'posedge clk' --eval "top.cpu.state == 8'h03"
```

Canonical paths must be inside the selected scope. Every other name is interpreted relative to that scope. For example, under `--scope top.cpu`, `other.clk` means `top.cpu.other.clk`; it does not select canonical path `other.clk` outside the scope.

The same rule applies to `value` and `change` `--signals`, names in `--on`, `--eval`, and `--when` expressions, and `extract generic --payload`.

For `value --signals`, `change --signals`, and generic payloads, append a flat projection after either naming form:

```text
wavepeek value --waves dump.vcd --at 10ns --scope top.cpu --signals state[7:4],top.cpu.data[31:16]
```

WavePeek scopes and resolves the base path, while output retains `[msb:lsb]`. Use `[n:n]` for one bit; `[n]` remains part of ordinary waveform path syntax.

## How to recover quickly

1. Use `wavepeek scope` to confirm the exact scope path.
2. Use `wavepeek signal --scope <path>` to confirm signals inside it.
3. Check that every canonical reference begins with the selected scope followed by `.`.
4. Check that every other reference exists relative to the selected scope.

If a query still fails, the scope path or signal spelling is usually wrong, or the signal lives outside the selected scope.
