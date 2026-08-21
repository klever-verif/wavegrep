# Extract AMBA APB

Use `extract apb` to inspect APB Setup, waited Access, and completed Access states. Select one concrete APB Completer interface before mapping its signals.

## Which APB accesses occurred?

```text
wavepeek extract apb --waves dump.fst \
  --scope tb.dut.uart_apb \
  --profile apb4 \
  --map pclk=clk \
  --map presetn=reset_n \
  --include '^uart_' \
  --from 5us --to 6us
```

Selected fields from representative rows:

```text
@5010ns sample@5009ns [setup write] pwrite=1'h1 paddr=16'h0040 pwdata=32'h00000055 pstrb=4'h1
@5020ns sample@5019ns [access-complete write] pwrite=1'h1 paddr=16'h0040 pwdata=32'h00000055 pslverr=1'h0
@5100ns sample@5099ns [setup read] pwrite=1'h0 paddr=16'h0044
@5110ns sample@5109ns [access-complete read] pwrite=1'h0 paddr=16'h0044 prdata=32'h000000a5 pslverr=1'h0
```

The rows are independently sampled APB states. The extractor does not combine Setup and Access rows into transaction objects or decode register addresses.

## Which accesses waited?

Add `--include-wait` in mapped-PREADY mode:

```text
wavepeek extract apb --waves dump.fst \
  --scope tb.dut.uart_apb \
  --profile apb4 \
  --map pclk=clk \
  --include '^uart_' \
  --include-wait \
  --from 5us --to 6us
```

A separate `access-wait` row appears for each sampled Access cycle where `PREADY` is low. Omit the option when only Setup and completion matter.

## What if PREADY is not present?

Use `--pready-mode implicit-high` only for an interface that physically omits `PREADY`. Do not map `pready` or request wait rows in that mode.

Map one concrete `PSELx` signal as `psel`; do not use a regex that merges several Completer selects into one interface. Run `wavepeek help extract apb` for profile-specific mappings and all mode options.
