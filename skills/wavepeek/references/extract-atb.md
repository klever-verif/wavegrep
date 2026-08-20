# Extract AMBA ATB

Use `extract atb` to inspect raw ATB transfer, flush, and synchronization-request events on one trace interface.

## Which trace transfers occurred?

```text
wavepeek extract atb --waves dump.fst \
  --scope tb.dut.etm \
  --profile atb-c \
  --map atclk=trace_clk \
  --map atresetn=trace_reset_n \
  --include '^trace_(at|af|sync)' \
  --from 30us --to 31us
```

Selected fields from representative rows:

```text
@30050ns sample@30049ns [transfer] atbytes=2'h3 atdata=32'h44332211 atid=7'h10
@30200ns sample@30199ns [flush]
@30400ns sample@30399ns [sync-request]
```

A transfer row appears for a sampled `ATVALID` and `ATREADY` handshake. Its payload can contain the mapped `ATBYTES`, `ATDATA`, and `ATID` values. The extractor preserves the raw vectors and does not trim `ATDATA` or decode trace packets and triggers.

A flush row comes from the `AFVALID` and `AFREADY` handshake. A mapped `SYNCREQ` adds synchronization-request rows for ATB-B and ATB-C.

## What is required for a useful mapping?

Map `atclk` and at least one complete handshake pair:

- `atvalid` with `atready` for trace transfers;
- `afvalid` with `afready` for flush events.

Payload fields are optional, so a handshake-only transfer still produces a row. Map `atresetn` when reset should gate the sampled events.

When several event types match the same edge, rows are ordered as transfer, flush, then synchronization request. Repeated handshakes are preserved even when payload values do not change.

Run `wavepeek help extract atb` for profile selection and all supported mappings.
