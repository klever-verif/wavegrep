# APB scoreboard example

This example combines the independently sampled events from `wavepeek extract apb` into APB transactions. One Setup event is joined with its waited Access cycles and final Access completion.

The scoreboard prints the start and completion times, request fields, read or write data, error response, wait count, and final statistics.

## Run it

Select one concrete APB Completer interface and pipe JSONL into the script:

```sh
wavepeek extract apb \
  --waves dump.fst \
  --scope tb.dut.uart_apb \
  --profile apb4 \
  --map pclk=clk \
  --map presetn=reset_n \
  --include '^uart_' \
  --include-wait \
  --max unlimited --jsonl |
python3 examples/apb_scoreboard/apb_scoreboard.py
```

Output has one line per completed transaction:

```text
WRITE start=5ns end=20ns addr=8'h40 pprot=3'h2 wait_cycles=2 error=1'h0 data=8'hde strb=1'h1
READ start=25ns end=30ns addr=8'h44 pprot=3'h1 wait_cycles=0 error=1'h1 data=8'ha5
SUMMARY completed_reads=1 completed_writes=1 completed_unknown=0 incomplete=0 unmatched_transfers=0
```

`--include-wait` is required only when the wait-cycle count matters. Use `--max unlimited` so Wavepeek does not split the event stream before a completion.

## Read from a file

The script also accepts Wavepeek JSONL or its single JSON result envelope from a file:

```sh
wavepeek extract apb ... --include-wait --max unlimited --jsonl > apb.jsonl
python3 examples/apb_scoreboard/apb_scoreboard.py apb.jsonl
```

## Scope of the example

This is a compact reconstruction example, not a protocol checker. It assumes one selected APB interface and a Setup event before each Access completion. It does not validate signal stability, protocol state transitions, resets, or malformed input.

The Setup and Access pairing follows the APB operating states in Arm IHI 0024E section 4.1. See [Extract AMBA APB](../../references/extract-apb.md) for extraction details and [Machine output contract](../../references/machine-output.md) for the JSON shapes.
