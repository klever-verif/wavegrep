# Extract AMBA AHB

Use `extract ahb` to inspect manager-facing AHB-Lite or AHB5 address and data-phase events. The extractor tracks the pipeline relationship between an accepted address and its later completion.

## Which AHB transfers were accepted and completed?

```text
wavepeek extract ahb --waves dump.fst \
  --scope tb.dut.ahb_m \
  --profile ahb-lite \
  --map hclk=clk \
  --map hresetn=reset_n \
  --include '^m_ahb_' \
  --from 2us --to 3us
```

Selected fields from representative rows:

```text
@2050ns sample@2049ns [address nonseq read] htrans=2'h2 hwrite=1'h0 haddr=32'h00001000 hsize=3'h2
@2060ns sample@2059ns [data-complete read] hresp=1'h0 hrdata=32'hdeadbeef
@2070ns sample@2069ns [address nonseq write] htrans=2'h2 hwrite=1'h1 haddr=32'h00001004 hsize=3'h2
@2080ns sample@2079ns [data-complete write] hresp=1'h0 hwdata=32'h12345678
```

Address and completion rows are separate because AHB pipelines the current address phase with the previous transfer's data phase. The extractor does not join them into one transaction row or reconstruct bursts.

Map the manager-facing selected `HREADY`. Do not substitute a subordinate-local `HREADYOUT`.

## Where did a transfer stall?

Add `--include-stall` to emit one row for each cycle where a pending data phase remains active while `HREADY` is low:

```text
wavepeek extract ahb --waves dump.fst \
  --scope tb.dut.ahb_m \
  --profile ahb-lite \
  --map hclk=clk \
  --include '^m_ahb_' \
  --include-stall \
  --from 2us --to 3us
```

`--include-idle` and `--include-busy` expose those address slots when cycle-level detail is useful. They are omitted by default to keep the event table focused.

The pipeline walker warms up before `--from`, so a completion inside the selected range can refer to an address accepted earlier. Check `initial_data_phase` in the command context when the first row needs explanation.

Run `wavepeek help extract ahb` for the AHB5 profile, optional mappings, and event controls.
