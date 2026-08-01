---
id: workflows/extract-handshake
title: Extract handshakes from synchronous bus
description: Turn ready/valid clock edges into deterministic row output.
section: workflows
see_also:
  - commands/extract
  - commands/property
  - commands/value
  - reference/expression-language
---
# Extract handshakes from synchronous bus

Use `extract` commands when you need a compact table of transfer-like events from a waveform. Use `extract ahb` for pipelined AHB-Lite or AHB5 address, completion, reset, and synchronization events. Use `extract apb` for APB3, APB4, or APB5 Setup and Access states from Arm IHI 0024E Issue E. Use `extract atb` for AMBA ATB transfer, flush, and synchronization-request events. Use `extract axi` for AXI3, AXI4, AXI4-Lite, AXI5, AXI5-Lite, ACE, ACE-Lite, ACE5, ACE5-Lite, ACE5-LiteDVM, and ACE5-LiteACP ready/valid channels. Use `extract axistream` for one AXI4-Stream or AXI5-Stream interface. AXI5 and ACE5-LiteDVM can include DVM `ac` and `cr` transfers without `cd`; AXI5-Lite, ACE5-Lite, and ACE5-LiteACP use only the five base channels. Use `extract generic` for other protocol-neutral handshakes.

For AHB, map manager-facing `HREADY` together with the clock and address-phase controls. Do not substitute subordinate-local `HREADYOUT`:

```text
$ wavepeek extract ahb --waves path/to/dump.vcd \
    --scope top.dut \
    --profile ahb-lite \
    --map hclk=clk \
    --map hresetn=rst_n \
    --include '^ahb_'
name: ahb
profile: ahb-lite
issue: C
include_stall: false
include_idle: false
include_busy: false
initial_data_phase: desynchronized
mappings:
  hclk = clk
  hresetn = rst_n
  htrans = ahb_htrans
  hready = ahb_hready
events:
@25ns sample@24999ps [address nonseq read] htrans=2'h2 hwrite=1'h0 haddr=32'h00000040
@35ns sample@34999ps [data-complete read] hresp=1'h0 hrdata=32'hdeadbeef
```

The AHB walker retains a pending phase across low-`HREADY` cycles, emits a real completion when it advances, and warms state before `--from`. Add `--include-stall`, `--include-idle`, or `--include-busy` only when cycle-level rows are useful.

For APB, select the concrete Completer interface and optionally keep waited Access cycles:

```text
$ wavepeek extract apb --waves path/to/dump.vcd \
    --scope top.uart \
    --profile apb4 \
    --include '^uart_apb_' \
    --include-wait
name: apb
profile: apb4
issue: E
pready_mode: mapped
include_wait: true
mappings:
  pclk = uart_apb_pclk
  psel = uart_apb_psel
  penable = uart_apb_penable
  pwrite = uart_apb_pwrite
  pready = uart_apb_pready
  paddr = uart_apb_paddr
  pwdata = uart_apb_pwdata
  pslverr = uart_apb_pslverr
events:
@20ns sample@19ns [setup write] pwrite=1'h1 paddr=16'h0040 pwdata=32'hdeadbeef
@30ns sample@29ns [access-wait write] pwrite=1'h1 paddr=16'h0040 pwdata=32'hdeadbeef
@40ns sample@39ns [access-complete write] pwrite=1'h1 paddr=16'h0040 pwdata=32'hdeadbeef pslverr=1'h0
```

The APB rows above are independent sampled states, not assembled transactions. Omit `--include-wait` when only Setup and completion matter. Use `--pready-mode implicit-high` only when PREADY is physically absent; that mode forbids both a `pready` mapping and wait capture.


For AXI, map the clock and let include regexes find standard channel signals:

```text
$ wavepeek extract axi --waves path/to/dump.vcd \
    --scope top.dut \
    --profile axi4-lite \
    --map aclk=clk \
    --map aresetn=rst_n \
    --include '^axi_(aw|w|b|ar|r)_'
name: axi
profile: axi4-lite
issue: H.c
mappings:
  aclk = clk
  aresetn = rst_n
  awaddr = axi_aw_addr
  awvalid = axi_aw_valid
  awready = axi_aw_ready
transfers:
@25ns sample@24999ps [aw] awaddr=32'h00000040
```

For AXI-Stream, map one interface. The default mapped mode requires `tready`:

```text
$ wavepeek extract axistream --waves path/to/dump.vcd \
    --scope top.dut \
    --profile axi4-stream \
    --map aclk=clk \
    --map aresetn=rst_n \
    --include '^video_out_'
name: axistream
profile: axi4-stream
issue: B
tready_mode: mapped
mappings:
  aclk = clk
  aresetn = rst_n
  tvalid = video_out_tvalid
  tready = video_out_tready
  tdata = video_out_tdata
transfers:
@25ns sample@24999ps tdata=32'hdeadbeef
```

Use `--tready-mode implicit-high` only when the physical stream interface omits `TREADY`; do not map `tready` in that mode. The AXI-Stream adapter emits every handshake row, including identical consecutive payloads, and does not reconstruct packets from `tlast`.

For generic handshakes, start by selecting a scope and an edge-only event:

```text
$ wavepeek extract generic --waves path/to/dump.vcd \
    --scope top.dut \
    --on "posedge clk iff rst_n" \
    --when "valid && ready" \
    --payload data,last
@25ns sample@24999ps data=32'hdeadbeef last=1'h1
```

The event `time` is the clock edge. Payload values are sampled at `sample_time`, one dump tick before the edge.

For automation, prefer JSONL when the result may be large:

```text
$ wavepeek extract generic --waves path/to/dump.vcd \
    --scope top.dut \
    --on "posedge clk iff rst_n" \
    --when "valid && ready" \
    --payload data,last \
    --jsonl
```

For several generic source types, or for a reusable AHB, APB, ATB, AXI, or AXI-Stream profile/mapping setup, write a source file and pass `--source`. Use `wavepeek schema --input` to fetch the exact input schema for that file.
