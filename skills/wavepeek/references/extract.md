# Extract command

Use `extract` commands when you need row output that combines event selection, protocol state, predicate evaluation, or payload sampling. `extract ahb` follows the pipelined AHB address/data relationship. `extract apb` classifies APB Setup and Access states. `extract atb` expands AMBA ATB transfer, flush, and synchronization-request conditions into generic extraction sources. `extract axi` reports AXI3, AXI4, AXI4-Lite, AXI5, AXI5-Lite, ACE, ACE-Lite, ACE5, ACE5-Lite, ACE5-LiteDVM, and ACE5-LiteACP ready/valid channel transfers. `extract axistream` handles one AXI4-Stream or AXI5-Stream interface. `extract generic` is protocol-neutral.

For exact syntax and flags, run `wavepeek help extract <ahb|apb|atb|axi|axistream|generic>`.

## `extract ahb`

`extract ahb` emits deterministic manager-facing AHB pipeline events using Arm IHI 0033C, Issue C. The supported profiles are `ahb-lite` and `ahb5`; the default is `ahb-lite`. The CLI and source parser also accept `ahb_lite`, while source files also accept aliases.

AHB overlaps each transfer's address phase with the previous transfer's data phase. The extractor therefore keeps one pipeline slot instead of treating every high-`HREADY` clock as a completed transfer. At each rising `HCLK` edge it samples mapped values one dump tick before the edge, then:

1. if `HREADY` is high, completes a previously accepted data phase;
2. on that same edge, accepts current `HTRANS=NONSEQ` or `SEQ` into the next data-phase slot;
3. treats `IDLE` and `BUSY` as slots without a following data phase.

When a completion and a new address occur on the same edge, the `data-complete` row appears before the `address` row. A low `HREADY` extends a pending data phase; `--include-stall` emits one `data-stall` row for each such sampled cycle. `--include-idle` and `--include-busy` independently add address-slot rows. These optional rows are disabled by default.

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

The required standard mappings are `hclk`, `htrans`, `hready`, and `hwrite`. Optional AHB-Lite mappings are `hresetn`, `haddr`, `hburst`, `hmastlock`, `hprot`, `hsize`, `hauser`, `hwdata`, `hwstrb`, `hwuser`, `hrdata`, `hruser`, `hbuser`, and `hresp`. AHB5 additionally accepts `hnonsec`, `hexcl`, `hmaster`, and `hexokay`. Standard keys are lowercase.

Map signals explicitly with repeated `--map standard=waveform` options, auto-map candidates selected by repeated `--include REGEX`, or combine both. Explicit mappings override auto-mapping. Normalized full-suffix matching accepts common forms such as `haddr`, `h_addr`, `ahb_haddr_i`, and `ahb_h_addr_i`. It does not map `hreadyout` to `hready`, and it ignores parity/check lookalikes. With `--scope`, mapped waveform names and include regexes are scope-relative.

The command uses the manager-facing selected `HREADY`. It does not accept subordinate-local `HREADYOUT`, `HSELx`, or parity/check signals as standard mappings. It also does not reconstruct bursts, join address and data rows, assign transaction IDs, count stalls, mask byte lanes, validate protocol rules, or infer completions hidden by unknown history.

Mapped active-low `HRESETn` clears pending state. A consecutive known-low reset episode emits one `reset` boundary. Unknown reset, unknown `HREADY`, or unknown accepted `HTRANS` history moves the walker to `desynchronized`; one boundary is emitted when that state is entered. A later known-high `HREADY` edge establishes the next slot from current `HTRANS` without inventing an old completion. Unknown `HWRITE` preserves an accepted phase with `direction: "unknown"` rather than discarding it.

Before an inclusive `--from` bound, the walker warms from dump start without emitting or counting rows. Machine context records the resulting `initial_data_phase` as `empty`, `desynchronized`, or `pending`. A pending state retains the earlier accepted address snapshot so a later in-range completion has explicit provenance.

Address and BUSY rows contain mapped address/control observations. IDLE rows contain only signals valid for IDLE. A write stall or completion can contain write data, strobes, and write user data. A known-direction read stall does not claim read data validity. Read completion preserves mapped `hrdata`, including on ERROR. `hruser`, `hbuser`, and AHB5 `hexokay` appear only when mapped `hresp` is known low. Unknown direction preserves both mapped data sides as observations. X/Z values remain sized Verilog literals.

A source file can provide canonical or aliased `profile`, `name`, `include_stall`, `include_idle`, `include_busy`, `includes`, and `maps` with `kind: "extract.ahb.source"`. Source-file mode conflicts with the matching CLI configuration flags; time bounds and scope remain command-line options. The fields are described here and in command help.

Machine-readable AHB output is typed by profile and event. JSON uses `command: "extract ahb"`, carries Issue C metadata, inclusion flags, initial pipeline state, and canonical mappings in `context`, and puts ordered events directly in `data`. JSONL begins with the same context and emits one event per `data` record. Only emitted public events count toward `--max`, so a limit can stop between a same-edge completion/address pair.

## `extract apb`

`extract apb` emits independent sampled APB events for the `apb3`, `apb4`, and `apb5` profiles from Arm IHI 0024E Issue E. The default is APB4. At the pre-edge sample point for `posedge pclk`, a Setup event is `psel && !penable`. A completed Access is `psel && penable && pready` in mapped-PREADY mode or `psel && penable` in implicit-HIGH mode. Add `--include-wait` in mapped mode to emit one `access-wait` row per cycle where `psel && penable && !pready`. If `presetn` is mapped, every predicate is also gated by sampled known-HIGH reset.

Mapped mode is the default and requires `pready`. Implicit-HIGH mode forbids both a `pready` mapping and `--include-wait`. Unknown `psel`, `penable`, `pready`, or mapped `presetn` values do not classify as true. Setup classification does not depend on `pready`. The command preserves repeated events and does not require or remember a preceding Setup phase.

Map lowercase standard names explicitly, select candidates with include regexes, or combine both. Explicit maps win. Auto-mapping requires a complete normalized signal-name suffix, so forms such as `paddr`, `p_addr`, `apb_paddr_i`, and `apb_p_addr_i` match `paddr`, while `paddrchk`, `psel0`, and `pselx` do not. Map one concrete Completer select such as `uart_psel` to canonical `psel`; indexed selects are not discovered as one combined bus.

All profiles require `pclk`, `psel`, `penable`, and `pwrite`. APB3 accepts the base APB3 signals. APB4 adds `pprot` and `pstrb`. APB5 adds `pnse`, request/data/response user signals, and the APB4 set. `pwakeup` and APB5 check/parity signals are outside extraction. Widths, sparse-write meaning, user-field meaning, and APB protocol conformance are not validated.

Every event includes sampled `pwrite` and derives `direction` as `read`, `write`, or `unknown`. Request fields can appear on every event. Read data, error response, and response-user fields appear only on completion. Known reads omit write data/user fields, known writes omit read data/user fields, and unknown direction preserves available direction-specific observations. Sampled vectors and X/Z literals are emitted unchanged.

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

A source file can provide `profile`, `pready_mode`, `include_wait`, `name`, `includes`, and `maps` with `kind: "extract.apb.source"`. The parser accepts profile and PREADY-mode values case-insensitively and accepts `implicit_high` as an alias; source files accept canonical lowercase values and the documented aliases. Source-file mode conflicts with the corresponding CLI flags. Time bounds, scope, row limit, output mode, and absolute-path rendering remain command-line concerns.

APB extraction is stateless sampled-event classification. It does not assemble transactions, pair Setup with Access, count waits into transaction records, decode registers, infer one Completer from several selects, or validate protocol sequencing, stability, parity, or errors.

## `extract atb`

`extract atb` emits stateless AMBA ATB interface events using Arm IHI 0032C Issue C definitions. The supported profiles are `atb-a`, `atb-b`, and `atb-c`; the default is `atb-c`. The CLI and source parser also accept underscore aliases, and accept legacy `atbv1.0` and `atbv1.1` as aliases for ATB-A and ATB-B. Source files also accept the documented aliases.

Every configuration maps `atclk` and at least one complete handshake pair. A transfer event requires known-true `atvalid && atready`; a flush event requires known-true `afvalid && afready`. Mapping `syncreq` on ATB-B or ATB-C adds a synchronization-request source whose predicate is known-true `syncreq`, but does not replace the required transfer or flush pair. `atresetn` is optional; when mapped, each predicate is additionally gated by known-true `atresetn` at the pre-edge sample point. Unknown control or reset values do not produce the affected event.

Transfer payload mappings are optional and stay raw. A transfer row can include mapped `atbytes`, `atdata`, and `atid` values in that order. `ATBYTES + 1` is the number of valid low-order `ATDATA` bytes, but the command preserves the complete observed vectors without trimming or masking; upper bytes outside that count are observations and are not claimed as protocol-valid trace bytes. This permits handshake-only extraction and 8-bit `ATDATA` interfaces where `ATBYTES` is absent. Flush and synchronization-request rows have empty payloads. `ATID` values are observations rather than decoded trigger or protocol semantics.

The extraction profiles deliberately exclude `atclken` and `atwakeup`. ATB-A also excludes `syncreq`; ATB-B and ATB-C accept it. The initial ATB-B and ATB-C extraction signal sets are otherwise identical.

Map signals explicitly or select automatic candidates with include regexes. Matching is case-insensitive after separator normalization, accepts leading interface prefixes and common direction suffixes, and requires a complete standard-signal suffix. Explicit mappings win. With `--scope`, waveform mapping names and include candidates are relative to that scope.

```text
$ wavepeek extract atb --waves path/to/dump.vcd \
    --scope top.etm \
    --profile atb-c \
    --map atclk=trace_clk \
    --map atresetn=trace_reset_n \
    --include '^trace_(at|af|sync)'
name: atb
profile: atb-c
issue: C
mappings:
  atclk = trace_clk
  atresetn = trace_reset_n
  atvalid = trace_at_valid
  atready = trace_at_ready
  atbytes = trace_at_bytes
  atdata = trace_at_data
  atid = trace_at_id
  afvalid = trace_af_valid
  afready = trace_af_ready
  syncreq = trace_sync_req
events:
@25ns sample@24999ps [transfer] atbytes=2'h3 atdata=32'h44332211 atid=7'h10
@25ns sample@24999ps [flush]
@25ns sample@24999ps [sync-request]
```

When several event conditions are true at one edge, rows appear in `transfer`, `flush`, then `sync-request` order. Every sampled high `syncreq` produces a row independently, and repeated transfer or flush handshakes are preserved even when values do not change.

A source file can provide `profile`, `name`, `includes`, and `maps` with `kind: "extract.atb.source"`. Source-file mode conflicts with the corresponding command-line configuration options; time bounds and scope remain command-line settings.

ATB extraction does not reconstruct trace packets, derive byte counts, decode trace triggers, verify legal encodings, or infer cross-cycle transfer, flush, synchronization, or wake-up episodes. Use the raw event rows as evidence tied to their sampled edge.

Arm IHI 0032C sections 3.1-3.2 define ATB transfer sampling and the `ATVALID`/`ATREADY` handshake. Section 4.2 defines the flush handshake, section 4.4 defines synchronization requests, and Appendix A Table A-1 defines the interface signal matrix.


## `extract axi`

`extract axi` emits one row per completed AXI-family transfer on each mapped ready/valid channel. Supported profiles are `axi3`, `axi4`, `axi4-lite`, `axi5`, `axi5-lite`, `ace`, `ace-lite`, `ace5`, `ace5-lite`, `ace5-lite-dvm`, and `ace5-lite-acp`; the default profile is `axi4`. AXI3, AXI4, AXI4-Lite, ACE, ACE-Lite, and ACE5 use Arm IHI 0022H.c Issue H.c signal definitions. AXI5, AXI5-Lite, ACE5-Lite, ACE5-LiteDVM, and ACE5-LiteACP use Arm IHI 0022L Issue L ready/valid signal definitions. A completed transfer requires both channel `VALID` and channel `READY` to be true at the pre-edge sample point for `posedge aclk`. If `aresetn` is mapped, it must also be true at that sample point.

The CLI and source parser accept `ace5_lite` for ACE5-Lite. ACE5-LiteDVM additionally accepts `ace5-litedvm`, `ace5_litedvm`, and `ace5_lite_dvm`; ACE5-LiteACP additionally accepts `ace5-liteacp`, `ace5_liteacp`, and `ace5_lite_acp`. Source files also accept the documented aliases.

Map signals explicitly with repeated `--map standard=waveform` options, auto-map candidates selected by repeated `--include REGEX`, or combine both. Standard signal names are lowercase AXI names such as `awvalid`, `awready`, `wdata`, `rresp`, and `acvalid`; explicit mappings override auto-mapping for the same standard signal. With `--scope`, mapped waveform names and include regexes are scope-relative.

AXI5 and ACE5-LiteDVM add the `ac` and `cr` DVM channels after the base `aw`, `w`, `b`, `ar`, and `r` channels when those signals are mapped; neither adds a `cd` channel. AXI5-Lite, ACE5-Lite, and ACE5-LiteACP use only the five base channels. ACE and ACE5 add the `ac`, `cr`, and `cd` coherency channels. ACE-Lite uses only the five base channels and accepts its read/write address additions, including optional `awunique`. ACE5 does not accept the removed `awbar` or `arbar` signals. Optional and conditional payload signals are extracted when mapped and are not required.

AXI-family extraction reports functional ready/valid channel transfers only. The Issue L profiles do not accept credited transport signals. Extraction does not include standalone `rack` or `wack` acknowledgements, interface-level wakeup or coherency-connection signals, QoS-accept controls, or check/parity signals. It does not reconstruct bursts, ordering, DVM messages, or coherency state.

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

A source file can provide `profile`, `name`, `includes`, and `maps` with `kind: "extract.axi.source"`. Source-file mode conflicts with `--profile`, `--name`, `--map`, and `--include`; time bounds and scope still come from the command line.

Machine-readable AXI output is typed by profile and channel. JSON `data` rows and JSONL `data.data` rows include `profile`; payload keys depend on the selected profile and channel, and unmapped keys are omitted.

## `extract axistream`

`extract axistream` emits one row per completed transfer on one mapped stream interface. Its profiles are `axi4-stream` and `axi5-stream`; both use Arm IHI 0051B Issue B and the default is `axi4-stream`. The CLI and source parser accept profile names case-insensitively and accept the underscore aliases `axi4_stream` and `axi5_stream`. Source files also accept the documented aliases.

Map `aclk`, optional `aresetn`, handshake signals, and any payload signals with `--map`, `--include`, or both. Accepted payload standard names are `tdata`, `tstrb`, `tkeep`, `tlast`, `tid`, `tdest`, and `tuser`. Payload signals are optional and omitted from mappings and rows when unmapped. `twakeup` and check/parity signals are not part of transfer extraction.

The default `--tready-mode mapped` requires a `tready` mapping and recognizes a transfer when `tvalid && tready` is true at the pre-edge sample point for `posedge aclk`. Use `--tready-mode implicit-high` only when the physical interface omits `TREADY`; this mode forbids a `tready` mapping and recognizes transfers from `tvalid`. If `aresetn` is mapped, it gates either predicate. `aclk` and `tvalid` are always required.

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
  tlast = video_out_tlast
transfers:
@25ns sample@24999ps tdata=32'hdeadbeef tlast=1'h1
```

One invocation maps one interface. Rows do not contain a synthetic channel field, and a handshake-only transfer still emits its event and sample timestamps. The command does not reconstruct packets, interpret byte qualifiers, check protocol timing, or validate AXI5-Stream wake-up or parity.

A source file uses singular kind `extract.axistream.source` and can provide `profile`, `tready_mode`, `name`, `includes`, and `maps`. The defaults are `axi4-stream`, `mapped`, and `axistream`. Source-file mode conflicts with those CLI mapping/configuration options; time bounds and scope remain command-line options.

## `extract generic`

`extract generic` emits one row per matching synchronous event. It avoids the manual workflow of running `property`, extracting `sample_time` values, running `value`, and joining the results externally.

The command always samples at the pre-edge sample point. It does not support `--sample-mode`.

## Single-source extraction

A single-source query defines the source directly on the command line:

```text
$ wavepeek extract generic --waves path/to/dump.vcd \
    --scope top.dut \
    --on "posedge clk iff rst_n" \
    --when "valid && ready" \
    --payload data,last
@25ns sample@24999ps data=32'hdeadbeef last=1'h1
```

`--on` selects candidate event timestamps. `extract generic` only accepts edge-only event expressions, such as `posedge clk`, `negedge clk`, or `edge clk`, with optional `iff` gating. Wildcard triggers, plain signal triggers, and mixed level/edge triggers are rejected.

`--when` is a Boolean expression evaluated at the pre-edge sample point. `--payload` is the ordered list of signals sampled at the same pre-edge point. Payload entries may append one static decimal `[msb:lsb]` projection to a flat integral signal. Bit zero is the rightmost bit of the normalized sampled value, and `[n:n]` selects one bit. Exact waveform paths resolve first; `[n]` remains ordinary waveform path syntax. Duplicate, overlapping, projected, and complete-source payload entries are preserved in request order. The command emits a row only when the event matches and `--when` is true.

With `--scope`, signal references in `--on`, `--when`, and `--payload` may be relative or canonical paths inside the scope, and both forms may be mixed. The same rule applies to source-file fields.

## Source files

Use `--source` when one query should extract several source types from the same dump:

```json
{
  "kind": "extract.generic.sources",
  "sources": [
    {
      "name": "fifo.write",
      "on": "posedge wclk iff wrst_n",
      "when": "wvalid && wready",
      "payload": ["wdata", "wlast"]
    },
    {
      "name": "fifo.read",
      "on": "posedge rclk iff rrst_n",
      "when": "rvalid && rready",
      "payload": ["rdata"]
    }
  ]
}
```

Then run:

```text
$ wavepeek extract generic --waves path/to/dump.vcd --scope top.dut --source sources.json --jsonl
```

Source names must be unique within the file. Payload arrays accept the same projections as CLI `--payload` and preserve duplicate entries. Source-file mode conflicts with `--name`, `--on`, `--when`, and `--payload` because those fields come from the file. Source-file fields and behavior are described above and in command help.

## Pre-edge sampling

`extract` rows use `time` for the selected event timestamp and `sample_time` for the point where predicate and payload values are read. `sample_time` is one dump tick before the selected edge.

This matches common RTL debugging expectations: the row describes the values that caused the edge to be interesting, not values updated on the edge itself.

`--from` and `--to` bound event `time` values. A row at `--from` can still use a `sample_time` before `--from` if that sample point is inside the dump.

## Output modes

Human `extract ahb` output starts with name, profile, issue, inclusion flags, initial data-phase state, resolved mappings, and then event rows. Add `--abs` to print canonical mapping and payload paths. JSON and JSONL carry the full retained pending-address snapshot when `initial_data_phase` is `pending`.
Human `extract apb` output starts with name, profile, Issue E, PREADY mode, effective wait setting, resolved mappings, and then event rows. `extract apb --json` uses `command: "extract apb"`, puts metadata in `context`, and puts events directly in `data`; JSONL puts the same context on `begin` and one event on each `data` record. Profile, mode, wait setting, event, direction, mapping keys, and payload keys follow the documented APB contract. Add `--abs` to print canonical mapping and payload paths in human output.

Human `extract atb` output starts with name, profile, issue, resolved mappings, and then event rows. `extract atb --json` emits `command: "extract atb"`, puts name, profile, issue, and mappings in `context`, and puts events directly in `data`. JSONL puts the same context on `begin` and streams one event per `data` record. Add `--abs` to print canonical mapping and payload paths in human output.

Human `extract axi` output starts with name, profile, issue, resolved mappings, and then transfer rows. Add `--abs` to print canonical mapping and payload paths in human output.

`extract axi --json` emits the standard envelope with `command: "extract axi"`, puts name, profile, issue, and mappings in `context`, and puts transfers directly in `data`. `extract axi --jsonl` streams a `begin` record with the same AXI context and one transfer per `data` record.

`extract axistream` uses the same context-first layout plus `tready_mode`, but its rows have no `channel`. JSON uses `command: "extract axistream"`; JSONL puts name, profile, Issue B, TREADY mode, and mappings in the `begin` context and emits one independently profile-typed transfer per `data` record.

Human `extract generic` output is compact and row-oriented:

```text
@25ns sample@24999ps data=32'hdeadbeef last=1'h1
```

For multi-source output, the source name appears after `sample@...`:

```text
@25ns sample@24999ps [fifo.write] wdata=32'hdeadbeef wlast=1'h1
```

Add `--abs` to print canonical payload paths in human output.

`extract generic --json` emits the standard envelope with `command: "extract generic"` and an array of rows. With `--scope top`, the envelope contains `context: {"scope":"top"}` and each payload value contains canonical `path` plus scope-relative `relative_path`. Without `--scope`, `context` and `relative_path` are omitted while `path` remains canonical. `extract generic --jsonl` puts the same scope context in `begin` and streams `data`, `diagnostic`, and `end` records; each data row has `time`, `sample_time`, `source`, and ordered `payload` values.

Repeated events are preserved even when payload values do not change. `extract` is not a delta command. Add `--summary` to suppress event or transfer rows while retaining protocol context when present, completeness metadata, and diagnostics.

## Limits and diagnostics

For `extract generic`, `--max` limits emitted rows across all sources after sorting by event time and source declaration order. For `extract ahb`, it limits public event rows after warm-up and completion-before-address ordering. For `extract axi`, it limits ready/valid transfer rows. `--max unlimited` disables truncation without a diagnostic and appears as `summary.limit: null` in machine output. Empty results are successful and produce no machine diagnostic; generic human output prints a short message, while protocol human output retains its context and empty event or transfer section. Truncation still emits `WPK-W0002`.

Machine output includes `complete`, `returned`, `limit`, and `total` in the result summary. A bounded scan becomes incomplete only after another matching public event is found beyond the limit. Because truncated event scans stop early, their exact total is normally unknown.
