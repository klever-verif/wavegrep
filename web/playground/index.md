---
hide:
  - navigation
  - toc
---

<div class="playground" data-version="@WAVEPEEK_VERSION@">
  <h1 class="playground__visually-hidden">WavePeek Playground</h1>

  <section class="playground__install" aria-label="Install WavePeek">
    <a href="https://github.com/kleverhq/wavepeek/releases" target="_blank" rel="noopener noreferrer">Get WavePeek on GitHub Releases ↗</a>
    <span>or copy-paste this to your agent:</span>
    <div class="playground__install-prompt">
      <p id="agent-prompt" data-copy="Get latest WavePeek from https://github.com/kleverhq/wavepeek/releases. Use 'wavepeek skill' to get the skill.">Get latest WavePeek from https://github.com/kleverhq/wavepeek/releases. Use <code>'wavepeek skill'</code> to get the skill.</p>
      <span id="copy-status" class="playground__copy-status" aria-live="polite"></span>
      <button id="copy-agent-prompt" type="button">Copy</button>
    </div>
  </section>

  <div class="playground__toolbar">
    <nav class="playground__commands" aria-label="Example WavePeek commands">
      <span>Commands</span>
      <button type="button" data-example="info">Info</button>
      <button type="button" data-example="scope">Scope</button>
      <button type="button" data-example="signal">Signal</button>
      <button type="button" data-example="value">Value</button>
      <button type="button" data-example="change">Change</button>
      <button type="button" data-example="property">Property</button>
      <button type="button" data-example="extract">Extract AXI</button>
      <span class="playground__commands-more">and more with</span>
      <button type="button" data-example="help">Help</button>
    </nav>

    <section class="playground__source" aria-labelledby="source-heading">
      <h2 id="source-heading">Waveform source</h2>
      <button id="use-demo" type="button" class="md-button md-button--primary" aria-pressed="true">Demo</button>
      <div class="playground__local-source">
        <button id="open-local" type="button" class="md-button">Local VCD/FST</button>
        <input id="local-file" type="file" accept=".vcd,.fst" hidden>
      </div>
      <div class="playground__source-meta">
        <span id="source-indicator" class="playground__source-indicator" aria-hidden="true"></span>
        <div>
          <strong id="source-name">Loading demo…</strong>
          <p><span id="source-size">—</span> · <span id="source-format">FST</span> · <span id="source-status" aria-live="polite">Loading…</span></p>
        </div>
      </div>
      <a id="open-surfer" class="playground__surfer" target="_blank" rel="noopener noreferrer">Open visually in Surfer ↗</a>
      <p class="playground__source-privacy">Waveform data never leaves your browser. All processing happens locally.</p>
    </section>
  </div>

  <div class="playground__workspace">
    <section class="playground__terminal" aria-label="WavePeek terminal">
      <div class="playground__command-line">
        <label for="command-line"><span aria-hidden="true">$</span> wavepeek</label>
        <input id="command-line" type="text" spellcheck="false" autocapitalize="off" autocomplete="off" aria-describedby="command-error terminal-shortcuts">
        <button id="run" type="button" class="playground__run">Run</button>
        <button id="clear" type="button">Clear</button>
      </div>
      <p id="command-error" class="playground__error" aria-live="polite"></p>
      <div id="transcript" class="playground__transcript" role="log" aria-live="polite" tabindex="0">
        <p class="playground__empty">Run a command to see its output.</p>
      </div>
      <p id="terminal-shortcuts" class="playground__terminal-tip">Enter to run · ↑/↓ for command history · Ctrl+K to clear</p>
    </section>

    <aside class="playground__sidebar" aria-label="Playground options">
      <fieldset class="playground__modes">
        <legend>Output mode</legend>
        <div>
          <label><input type="radio" name="output-mode" value="human" checked><span>Human</span></label>
          <label><input type="radio" name="output-mode" value="json"><span>JSON</span></label>
          <label><input type="radio" name="output-mode" value="jsonl"><span>JSONL</span></label>
        </div>
        <p id="output-description">Human-readable output for exploration.</p>
      </fieldset>

      <section class="playground__suggestions" aria-labelledby="suggestions-heading">
        <h2 id="suggestions-heading">Example queries</h2>
        <div>
          <button type="button" data-suggestion="info">What is in this waveform?</button>
          <button type="button" data-suggestion="scope">What scopes are under the testbench?</button>
          <button type="button" data-suggestion="signal">Which AXI address signals were dumped?</button>
          <button type="button" data-suggestion="value">What were the AXI controls at 1000 ps?</button>
          <button type="button" data-suggestion="property">When did AXI read handshakes occur?</button>
          <button type="button" data-suggestion="extract">Which AXI transactions occurred?</button>
          <button type="button" data-suggestion="change">How did ARVALID and ARREADY change?</button>
          <button type="button" data-suggestion="generic">Which read addresses were transferred?</button>
        </div>
      </section>

    </aside>
  </div>
</div>

<script type="module" src="assets/playground/playground.js"></script>
