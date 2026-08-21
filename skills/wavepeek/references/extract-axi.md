# Extract AMBA AXI

Use `extract axi` to inspect accepted transfers on AXI-family ready/valid channels. Choose the profile that matches the interface, map its clock, and let an include regex find channel signals with regular names.

## What AXI traffic occurred in this range?

```text
wavepeek extract axi --waves dump.fst \
  --scope tb.dut.axi_m \
  --profile axi4 \
  --map aclk=clk \
  --map aresetn=reset_n \
  --include '^m_axi_(aw|w|b|ar|r)' \
  --from 10us --to 11us
```

Selected fields from representative transfer rows:

```text
@10050ns sample@10049ns [aw] awid=4'h2 awaddr=32'h00004000 awlen=8'h03
@10060ns sample@10059ns [w] wdata=32'hdeadbeef wstrb=4'hf wlast=1'h0
@10110ns sample@10109ns [b] bid=4'h2 bresp=2'h0
@10400ns sample@10399ns [ar] arid=4'h5 araddr=32'h00008000 arlen=8'h00
@10430ns sample@10429ns [r] rid=4'h5 rdata=32'h12345678 rresp=2'h0 rlast=1'h1
```

Each row is one accepted channel transfer. The extractor does not join AW, W, and B rows into writes or AR and R rows into reads.

## Which write addresses started here?

Restrict auto-mapping to the AW channel:

```text
wavepeek extract axi --waves dump.fst \
  --scope tb.dut.axi_m \
  --profile axi4 \
  --map aclk=clk \
  --include '^m_axi_aw' \
  --from 40us --to 50us
```

An AW row means that a write address was accepted. It does not mean that write data or the write response completed. Use the W and B channels when those stages matter.

## What if automatic mapping misses a signal?

Keep the include regex narrow enough to select one interface. Add explicit mappings for irregular names:

```text
--map awvalid=write_address_valid \
--map awready=write_address_ready \
--map awaddr=write_address
```

Explicit mappings override auto-mapped candidates. The resolved mapping printed before the rows is worth checking before interpreting an empty result.

AXI extraction reports channel transfers, not bursts, ordering, outstanding requests, DVM messages, or coherency state. Run `wavepeek help extract axi` for the supported profiles and mapping options.
