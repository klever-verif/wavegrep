# Boolean conditions

This page describes the boolean expressions that `wavepeek` accepts with `property --eval` and `extract generic --when` and evaluates over sampled waveform values. An `iff` guard in an event expression uses the same expression language.

The language is a SystemVerilog-compatible subset. Signal references keep the types and widths recovered from the dump, including four-state `x` and `z` values where the operation supports them.

See [Boolean expression language contract](boolean-expressions.md) for operand types, casts, conversions, operators, precedence, and grammar.

Simple conditions work as expected:

```text
valid && ready
state == 3
count >= 8'd10
```

Expressions can also use arithmetic, bitwise and logical operators, shifts, bit-selects, part-selects, concatenation, and replication:

```text
(count + 1) == limit
data[7:4] == 4'ha
(data & mask) != 0
{header, payload[3:0]} == expected
value == {4{2'b10}}
```

The full language also includes casts, reductions, `inside`, the `?:` conditional operator, and SystemVerilog equality forms such as `===` and `==?`.

## Literals

Integral literals use SystemVerilog syntax:

```text
12
'd12
'hff
8'hff
16'sd12
4'b10xz
```

C-style literals such as `0xff` and `0b1010` are not supported. Use `8'hff` and `4'b1010` instead.
