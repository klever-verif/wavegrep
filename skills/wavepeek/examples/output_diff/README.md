# Wavepeek output diff example

This example compares the result rows from the same Wavepeek query before and after a design change. It reports where the row streams first diverge and can show every later insertion, deletion, or replacement.

The script compares Wavepeek output rather than waveform files. It accepts raw `--json` or `--jsonl` files from `change`, `property`, `extract`, or another command that returns `data` rows.

## Capture both results

Run the same bounded query against both dumps. Use `--max unlimited` so a shared row limit cannot hide later differences:

```sh
wavepeek change --waves before.fst \
  --scope tb.dut.cpu \
  --from 10us --to 11us \
  --on 'posedge clk iff reset_n' \
  --signals state,pc,valid \
  --max unlimited --json > before.json

wavepeek change --waves after.fst \
  --scope tb.dut.cpu \
  --from 10us --to 11us \
  --on 'posedge clk iff reset_n' \
  --signals state,pc,valid \
  --max unlimited --json > after.json
```

## Find the first divergence

```sh
python3 examples/output_diff/output_diff.py before.json after.json
```

Example output:

```text
DIFF 1 replace
  left  rows=17:18 time=10240ns sample_time=10239ns
  right rows=17:18 time=10240ns sample_time=10239ns
- {"sample_time":"10239ns","signals":[{"path":"tb.dut.cpu.state","relative_path":"state","value":"4'h3"},{"path":"tb.dut.cpu.pc","relative_path":"pc","value":"32'h00001040"},{"path":"tb.dut.cpu.valid","relative_path":"valid","value":"1'h1"}],"time":"10240ns"}
+ {"sample_time":"10239ns","signals":[{"path":"tb.dut.cpu.state","relative_path":"state","value":"4'h4"},{"path":"tb.dut.cpu.pc","relative_path":"pc","value":"32'h00001040"},{"path":"tb.dut.cpu.valid","relative_path":"valid","value":"1'h1"}],"time":"10240ns"}
SUMMARY equal=false left_rows=64 right_rows=64 differing_blocks=3 removed_rows=3 added_rows=3 shown_blocks=1
```

Row ranges are zero-based and half-open. Run with `--all` to print all differing blocks:

```sh
python3 examples/output_diff/output_diff.py before.json after.json --all
```

The process exits with status 0 when the row streams are equal and 1 when they differ or an input cannot be compared.

## How matching works

The script removes the JSON/JSONL envelope and compares canonical `data` rows with Python's standard `difflib.SequenceMatcher`. JSONL `seq`, context, diagnostics, and summaries do not participate in row matching. Inserted or removed rows therefore do not shift every later row in the report.

Both files must come from the same Wavepeek command. Results with `complete: false`, JSONL streams without a final `end`, and `fatal` records are rejected. The script intentionally does not decide whether a difference is correct; extend `canonical()` when a project needs to ignore fields or normalize values. See the [machine output contract](../../references/machine-output.md) for the input shapes.

For an exact line-by-line comparison with no semantic alignment, use the system tool directly:

```sh
diff -u before.jsonl after.jsonl
```
