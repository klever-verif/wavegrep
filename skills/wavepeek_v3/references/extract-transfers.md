# Extract transfers

Use `extract generic` when a synchronous event can be described by a clock edge, a Boolean predicate, and a payload signal list. It keeps the predicate and payload on the same sampled row, so no external join between `property` and `value` is needed.

## Which handshakes occurred?

For a ready/valid interface:

```text
$ wavepeek extract generic --waves dump.fst \
    --scope tb.dut.queue \
    --on 'posedge clk iff reset_n' \
    --when 'valid && ready' \
    --payload data,last
@250ns sample@249ns data=32'hdeadbeef last=1'h0
@270ns sample@269ns data=32'hcafebabe last=1'h1
```

`--on` selects the clock edge. `--when` and `--payload` are evaluated immediately before that edge. The command emits one row only when the predicate is true.

The same pattern works for enables, FIFO pushes and pops, request/acknowledge pairs, and other clocked events:

```text
wavepeek extract generic --waves dump.fst \
  --scope tb.dut.fifo \
  --on 'posedge clk iff reset_n' \
  --when 'write_enable && !full' \
  --payload write_data,write_ptr
```

## How do I extract several event types at once?

Put the sources in a JSON file when they use different clocks, predicates, or payloads. For example, `fifo-sources.json`:

```json
{
  "kind": "extract.generic.sources",
  "sources": [
    {
      "name": "fifo.write",
      "on": "posedge write_clk iff write_reset_n",
      "when": "write_valid && write_ready",
      "payload": ["write_data", "write_last"]
    },
    {
      "name": "fifo.read",
      "on": "posedge read_clk iff read_reset_n",
      "when": "read_valid && read_ready",
      "payload": ["read_data"]
    }
  ]
}
```

Run both sources against the same scope:

```text
$ wavepeek extract generic --waves dump.fst \
    --scope tb.dut.async_fifo \
    --source fifo-sources.json
@410ns sample@409ns [fifo.write] write_data=32'h12345678 write_last=1'h0
@465ns sample@464ns [fifo.read] read_data=32'h12345678
```

Source names identify rows when several source definitions are active. Keep them short and specific.

## Should I use a generic or protocol extractor?

Individual events from any synchronous protocol can be described manually with generic predicates and payload lists. Stateful or pipelined bookkeeping must then be handled as separate low-level events or outside `wavepeek`. For a standard protocol, prefer its dedicated extractor. It supplies the channel or phase predicates, maps standard signal names, and handles protocol-specific bookkeeping that would otherwise become repetitive JSON.

- Use [Extract AMBA AXI](extract-axi.md) for AXI-family channel transfers.
- Use [Extract AMBA AXI-Stream](extract-axis.md) for stream beats.
- Use [Extract AMBA AHB](extract-ahb.md) for pipelined address and data-phase events.
- Use [Extract AMBA APB](extract-apb.md) for Setup and Access states.
- Use [Extract AMBA ATB](extract-atb.md) for trace transfers, flushes, and synchronization requests.

`extract generic` accepts edge-only event expressions and always uses pre-edge sampling. See [Clocks and sampling](sampling.md) and [Boolean conditions](predicates.md) for the shared semantics.
