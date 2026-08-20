# AHB scoreboard example

This example combines the pipelined events from `wavepeek extract ahb` into AHB transfers. An accepted address phase is joined with its data stalls and final data completion.

The scoreboard prints the address and data phase times, transfer attributes, read or write data, response, wait count, and final statistics.

## Run it

Select one manager-facing AHB interface and pipe JSONL into the script:

```sh
wavepeek extract ahb \
  --waves dump.fst \
  --scope tb.dut.ahb_m \
  --profile ahb-lite \
  --map hclk=clk \
  --map hresetn=reset_n \
  --include '^m_ahb_' \
  --include-stall \
  --max unlimited --jsonl |
python3 examples/ahb_scoreboard/ahb_scoreboard.py
```

Output has one line per completed transfer:

```text
READ start=15ns end=30ns transfer=nonseq addr=32'h00001000 size=3'h2 burst=3'h0 wait_cycles=2 resp=1'h1 data=32'hdeadbeef
WRITE start=35ns end=40ns transfer=nonseq addr=32'h00002000 size=3'h2 burst=3'h0 wait_cycles=0 resp=1'h0 data=32'ha5a55a5a strb=4'h5
SUMMARY completed_reads=1 completed_writes=1 completed_unknown=0 incomplete=0 unmatched_transfers=0
```

`--include-stall` is required only when the wait-cycle count matters. Use `--max unlimited` so Wavepeek does not split the phase stream before a completion.

The script also uses `initial_data_phase` from the Wavepeek context. This lets a completion after `--from` join an address phase accepted before the selected window.

## Read from a file

The script accepts Wavepeek JSONL or its single JSON result envelope from a file:

```sh
wavepeek extract ahb ... --include-stall --max unlimited --jsonl > ahb.jsonl
python3 examples/ahb_scoreboard/ahb_scoreboard.py ahb.jsonl
```

## Scope of the example

This is a compact reconstruction example, not a protocol checker. It emits one result per address and data-phase pair; it does not assemble those transfers into bursts. It also does not validate responses, ordering rules, resets, or malformed input.

The phase pairing follows the pipelined transfer model in Arm IHI 0033C section 3.1. See [Extract AMBA AHB](../../references/extract-ahb.md) for extraction details and [Machine output contract](../../references/machine-output.md) for the JSON shapes.
