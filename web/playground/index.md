---
hide:
  - navigation
  - toc
---

<div class="playground" data-version="@WAVEPEEK_VERSION@">
  <header class="playground__intro">
    <div>
      <p class="playground__eyebrow">WavePeek @WAVEPEEK_VERSION@</p>
      <h1>Inspect waveforms in your browser</h1>
      <p>Run the real WavePeek CLI against the bundled AXI demo or a local VCD/FST. Local files stay in this tab.</p>
    </div>
    <div class="playground__privacy" role="note">
      <strong>Local by design</strong>
      <span>No upload, account, backend, telemetry, or persistence.</span>
    </div>
  </header>

  <section class="playground__source" aria-labelledby="source-heading">
    <div>
      <h2 id="source-heading">Waveform</h2>
      <p id="source-status" class="playground__status" aria-live="polite">Loading bundled demo…</p>
    </div>
    <div class="playground__actions">
      <button id="use-demo" type="button" class="md-button">Use bundled demo</button>
      <label for="local-file" class="md-button md-button--primary">Choose local VCD/FST</label>
      <input id="local-file" type="file" accept=".vcd,.fst" hidden>
      <a id="open-surfer" class="md-button" target="_blank" rel="noopener noreferrer">Open demo in Surfer</a>
    </div>
    <p id="surfer-note" class="playground__hint">Surfer opens the same public demo for visual exploration.</p>
  </section>

  <div class="playground__workspace">
    <section class="playground__panel playground__controls" aria-labelledby="controls-heading">
      <h2 id="controls-heading">Command</h2>

      <label for="example">Example</label>
      <select id="example"></select>

      <div class="playground__control-grid">
        <label>Command
          <select id="command-kind">
            <option value="info">info</option>
            <option value="scope">scope</option>
            <option value="signal">signal</option>
            <option value="value">value</option>
            <option value="change">change</option>
            <option value="property">property</option>
            <option value="extract generic">extract generic</option>
            <option value="extract ahb">extract ahb</option>
            <option value="extract apb">extract apb</option>
            <option value="extract atb">extract atb</option>
            <option value="extract axi">extract axi</option>
            <option value="extract axistream">extract axistream</option>
          </select>
        </label>
        <label>Output
          <select id="output-mode">
            <option value="human">human</option>
            <option value="json">JSON</option>
            <option value="jsonl">JSONL</option>
          </select>
        </label>
        <label>Scope <input id="scope" type="text" data-option="--scope"></label>
        <label>Signals <input id="signals" type="text" data-option="--signals"></label>
        <label>At <input id="at" type="text" data-option="--at"></label>
        <label>From <input id="from" type="text" data-option="--from"></label>
        <label>To <input id="to" type="text" data-option="--to"></label>
        <label>On <input id="on" type="text" data-option="--on"></label>
        <label>Eval <input id="eval" type="text" data-option="--eval"></label>
        <label>When <input id="when" type="text" data-option="--when"></label>
        <label>Payload <input id="payload" type="text" data-option="--payload"></label>
        <label>Include <input id="include" type="text" data-option="--include"></label>
        <label>Max <input id="max" type="text" inputmode="numeric" data-option="--max"></label>
      </div>

      <label for="command-line">Editable CLI command</label>
      <textarea id="command-line" rows="5" spellcheck="false" autocapitalize="off"></textarea>
      <p id="command-error" class="playground__error" aria-live="polite"></p>

      <div class="playground__actions">
        <button id="run" type="button" class="md-button md-button--primary">Run</button>
        <button id="stop" type="button" class="md-button" disabled>Stop</button>
      </div>
    </section>

    <section class="playground__panel playground__result" aria-labelledby="result-heading">
      <div class="playground__result-heading">
        <h2 id="result-heading">Result</h2>
        <output id="exit-status" class="playground__exit" aria-live="polite">Not run</output>
      </div>
      <h3>stdout</h3>
      <pre id="stdout" tabindex="0">Run a command to see output.</pre>
      <h3>stderr</h3>
      <pre id="stderr" tabindex="0"></pre>
    </section>
  </div>

  <section class="playground__history" aria-labelledby="history-heading">
    <h2 id="history-heading">History</h2>
    <ol id="history"><li class="playground__empty">Commands run in this tab appear here.</li></ol>
  </section>
</div>

<script type="module" src="assets/playground/playground.js"></script>
