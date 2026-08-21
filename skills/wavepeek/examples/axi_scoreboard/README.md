# AXI scoreboard example

This example turns the channel transfers from `wavepeek extract axi` into completed AXI read and write transactions. It demonstrates the protocol layer that a consumer can build above Wavepeek's transfer-level output.

The scoreboard:

- starts reads on AR, associates R beats by `RID`, and completes them on `RLAST`;
- starts writes on AW, assigns W beats in AXI4 address order, and associates B responses by `BID`;
- prints transaction timing, address attributes, data beats, and responses;
- reports unmatched transfers, incomplete transactions, and completion counts.

## Run it

Select one concrete AXI interface and pipe JSONL into the script:

```sh
wavepeek extract axi \
  --waves dump.fst \
  --scope tb.dut.axi_m \
  --profile axi4 \
  --include '^m_axi_(aw|w|b|ar|r)' \
  --map aclk=clk \
  --map aresetn=reset_n \
  --max unlimited --jsonl |
python3 examples/axi_scoreboard/axi_scoreboard.py
```

Output has one line per completed transaction:

```text
WRITE start=472ps end=482ps id=4'h1 addr=32'h00000800 len=8'h00 size=3'h2 burst=2'h1 beats=1 resp=2'h0 data=[32'h00000000] strb=[4'hf]
READ start=562ps end=572ps id=4'h0 addr=32'h000008b0 len=8'h00 size=3'h0 burst=2'h1 beats=1 resp=[2'h0] data=[32'h00000054]
SUMMARY completed_reads=2 completed_writes=4 incomplete_reads=0 incomplete_writes=0 unmatched_transfers=0
```

Use `--max unlimited` so Wavepeek does not truncate the transfer stream. A time window that starts or ends during a transaction naturally produces unmatched or incomplete warnings.

## Read from a file

The script accepts either Wavepeek JSONL or its single JSON result envelope:

```sh
wavepeek extract axi ... --max unlimited --jsonl > axi.jsonl
python3 examples/axi_scoreboard/axi_scoreboard.py axi.jsonl
```

## Scope of the example

This is a compact reconstruction example, not a protocol checker. It assumes one AXI4 interface, AW appears before its W transfers in the selected stream, and `WLAST` and `RLAST` are mapped. It does not validate burst lengths, responses, ordering rules, resets, or malformed input.

AXI4 has no write-data ID, so W beats are assigned in write-address order rather than by `AWID`. This follows the AXI4 ordering model in Arm IHI 0022H.c sections A5.2.1 and A5.2.2. See [Extract AMBA AXI](../../references/extract-axi.md) for Wavepeek extraction and [Machine output contract](../../references/machine-output.md) for the JSON shapes.
