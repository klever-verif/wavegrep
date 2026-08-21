# Extract AMBA AXI-Stream

Use `extract axistream` to inspect transferred AXI-Stream beats. One invocation maps one stream interface.

## Which stream beats transferred?

```text
wavepeek extract axistream --waves dump.fst \
  --scope tb.dut.video_out \
  --profile axi4-stream \
  --map aclk=clk \
  --map aresetn=reset_n \
  --include '^video_' \
  --from 20us --to 21us
```

Selected fields from representative rows:

```text
@20050ns sample@20049ns tdata=32'h10203040 tkeep=4'hf tlast=1'h0
@20060ns sample@20059ns tdata=32'h50607080 tkeep=4'hf tlast=1'h1
```

A row appears when the mapped `TVALID` and `TREADY` handshake completes. Consecutive rows are preserved even when their payload values are identical.

`TLAST` is reported as a sampled payload field. The extractor does not group beats into packets or interpret `TKEEP`, `TSTRB`, `TID`, `TDEST`, or `TUSER`.

## What if the interface has no TREADY signal?

Use implicit-high mode only when `TREADY` is physically omitted:

```text
wavepeek extract axistream --waves dump.fst \
  --scope tb.dut.trace_stream \
  --profile axi4-stream \
  --tready-mode implicit-high \
  --map aclk=clk \
  --include '^trace_' \
  --from 5us --to 6us
```

Do not map `tready` in this mode. Every sampled high `TVALID` is treated as a transfer when mapped `ARESETn` is also high.

## What if automatic mapping finds the wrong interface?

Use a scope and an interface-specific include regex. Explicit mappings can fix irregular names:

```text
--map tvalid=out_valid --map tready=out_ready --map tdata=out_payload
```

Check the resolved mapping printed before the rows. Run `wavepeek help extract axistream` for all payload mappings and profile options.
