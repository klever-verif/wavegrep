const DEMO_NAME = "scr1_axi.fst";
const DEMO_URL = new URL("./scr1_axi.fst", import.meta.url);
const WORKER_URL = new URL("./worker.js", import.meta.url);
const HISTORY_LIMIT = 50;

const examples = {
  info: `info --waves ${DEMO_NAME}`,
  scope: `scope --waves ${DEMO_NAME} --tree --max-depth 3 --max 80`,
  signal: `signal --waves ${DEMO_NAME} --scope TOP.scr1_top_tb_axi.i_top --filter '.*io_axi_dmem_(arvalid|arready|araddr).*' --max 40`,
  value: `value --waves ${DEMO_NAME} --scope TOP.scr1_top_tb_axi.i_top --signals clk,io_axi_dmem_arvalid,io_axi_dmem_arready --at 1000ps`,
  change: `change --waves ${DEMO_NAME} --scope TOP.scr1_top_tb_axi.i_top --signals io_axi_dmem_arvalid,io_axi_dmem_arready --on 'posedge clk' --from 1ps --to 1880182ps --max 10`,
  property: `property --waves ${DEMO_NAME} --scope TOP.scr1_top_tb_axi.i_top --on 'posedge clk' --eval 'io_axi_dmem_arvalid && io_axi_dmem_arready' --capture match --from 1ps --to 1880182ps --max 10`,
  generic: `extract generic --waves ${DEMO_NAME} --scope TOP.scr1_top_tb_axi.i_top --on 'posedge clk' --when 'io_axi_dmem_arvalid && io_axi_dmem_arready' --payload io_axi_dmem_araddr,io_axi_dmem_arlen --from 1ps --to 1880182ps --max 10`,
  extract: `extract axi --waves ${DEMO_NAME} --scope TOP.scr1_top_tb_axi.i_top --include '^io_axi_dmem_' --map aclk=clk --map aresetn=axi_rst_n --from 1ps --to 1880182ps --max 10`,
};

const commandHelp = {
  info: "info --help",
  scope: "scope --help",
  signal: "signal --help",
  value: "value --help",
  change: "change --help",
  property: "property --help",
  extract: "extract --help",
  help: "help",
};

const elements = Object.fromEntries(
  [
    "source-name", "source-size", "source-format", "source-status", "source-indicator",
    "agent-prompt", "copy-agent-prompt", "copy-status", "use-demo", "open-local",
    "local-file", "open-surfer", "command-line", "command-error", "run", "clear",
    "transcript",
  ].map((id) => [id, document.getElementById(id)]),
);
const outputModes = [...document.querySelectorAll('input[name="output-mode"]')];

let activeSource;
let worker;
let runningId = 0;
let runningCommand = "";
let runningEntry;
let runningStarted = 0;
let commandHistory = [];
let historyIndex = null;
let historyDraft = "";
let sourceGeneration = 0;

export function tokenize(command) {
  const tokens = [];
  let token = "";
  let quote = "";
  let escaped = false;
  let started = false;

  for (const character of command) {
    if (escaped) {
      token += character;
      escaped = false;
      started = true;
    } else if (character === "\\" && quote !== "'") {
      escaped = true;
      started = true;
    } else if (quote) {
      if (character === quote) quote = "";
      else token += character;
    } else if (character === "'" || character === '"') {
      quote = character;
      started = true;
    } else if (/\s/.test(character)) {
      if (started) {
        tokens.push(token);
        token = "";
        started = false;
      }
    } else {
      token += character;
      started = true;
    }
  }
  if (escaped) throw new Error("Command ends with an incomplete escape");
  if (quote) throw new Error(`Command has an unmatched ${quote} quote`);
  if (started) tokens.push(token);
  return tokens;
}

function quoteArgument(argument) {
  return /^[A-Za-z0-9_./,:+=@%^-]+$/.test(argument)
    ? argument
    : `"${argument.replaceAll("\\", "\\\\").replaceAll('"', '\\"')}"`;
}

function renderCommand(tokens) {
  return tokens.map(quoteArgument).join(" ");
}

function setOption(tokens, option, value) {
  let insertion = tokens.length;
  for (let index = tokens.length - 1; index >= 1; index -= 1) {
    if (tokens[index] === option) {
      insertion = index;
      tokens.splice(index, index + 1 < tokens.length ? 2 : 1);
    } else if (tokens[index].startsWith(`${option}=`)) {
      insertion = index;
      tokens.splice(index, 1);
    }
  }
  if (value) tokens.splice(Math.min(insertion, tokens.length), 0, option, value);
}

function currentTokens() {
  return ["wavepeek", ...tokenize(elements["command-line"].value)];
}

function writeTokens(tokens) {
  elements["command-line"].value = renderCommand(tokens.slice(1));
  synchronizeOutputMode();
}

function synchronizeCommandSelection() {
  let selected;
  try {
    [selected] = tokenize(elements["command-line"].value);
    if (selected === "--help") selected = "help";
  } catch {
    selected = undefined;
  }
  for (const button of document.querySelectorAll("[data-example]")) {
    button.setAttribute("aria-pressed", String(button.dataset.example === selected));
  }
}

function synchronizeOutputMode() {
  synchronizeCommandSelection();
  try {
    const tokens = currentTokens();
    const value = tokens.includes("--jsonl") ? "jsonl" : tokens.includes("--json") ? "json" : "human";
    outputModes.find((input) => input.value === value).checked = true;
    elements["command-error"].textContent = "";
  } catch (error) {
    elements["command-error"].textContent = error.message;
  }
}

function selectOutput(value) {
  try {
    const tokens = currentTokens().filter((token) => token !== "--json" && token !== "--jsonl");
    if (value !== "human") tokens.push(`--${value}`);
    writeTokens(tokens);
  } catch (error) {
    elements["command-error"].textContent = error.message;
  }
}

function installExample(name) {
  const mode = outputModes.find((input) => input.checked).value;
  const tokens = ["wavepeek", ...tokenize(examples[name])];
  if (mode !== "human") tokens.push(`--${mode}`);
  writeTokens(tokens);
  elements["command-line"].focus();
}

function installCommandHelp(name) {
  writeTokens(["wavepeek", ...tokenize(commandHelp[name])]);
  elements["command-line"].focus();
}

function formatBytes(bytes) {
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KiB`;
  return `${(bytes / 1024 / 1024).toFixed(2)} MiB`;
}

function formatDuration(milliseconds) {
  return milliseconds < 1000 ? `${Math.round(milliseconds)} ms` : `${(milliseconds / 1000).toFixed(2)} s`;
}

function setRunning(running) {
  elements.run.textContent = running ? "Stop" : "Run";
  elements.run.dataset.running = String(running);
}

function stopWorker() {
  if (worker) worker.terminate();
  worker = undefined;
  setRunning(false);
}

function setSource(name, bytes, kind) {
  if (runningEntry) stopCommand();
  else stopWorker();
  activeSource = { name, bytes: new Uint8Array(bytes), kind };
  elements["source-name"].textContent = name;
  elements["source-size"].textContent = formatBytes(bytes.byteLength);
  elements["source-format"].textContent = name.split(".").pop().toUpperCase();
  elements["source-status"].textContent = "Ready";
  elements["source-indicator"].dataset.status = "ready";
  elements["use-demo"].setAttribute("aria-pressed", String(kind === "demo"));
  elements["use-demo"].classList.toggle("md-button--primary", kind === "demo");
  elements["open-surfer"].hidden = kind !== "demo";
  const tokens = currentTokens();
  if (tokens[1] !== "help" && !tokens.includes("--help")) setOption(tokens, "--waves", name);
  writeTokens(tokens);
}

async function useDemo() {
  const generation = ++sourceGeneration;
  elements["source-name"].textContent = DEMO_NAME;
  elements["source-size"].textContent = "—";
  elements["source-format"].textContent = "FST";
  elements["source-status"].textContent = "Loading…";
  elements["source-indicator"].dataset.status = "loading";
  try {
    const response = await fetch(DEMO_URL);
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    const bytes = await response.arrayBuffer();
    if (generation === sourceGeneration) setSource(DEMO_NAME, bytes, "demo");
  } catch (error) {
    if (generation === sourceGeneration) {
      elements["source-status"].textContent = `Could not load: ${error.message}`;
      elements["source-indicator"].dataset.status = "error";
    }
  }
}

function ensureWorker() {
  if (worker) return worker;
  worker = new Worker(WORKER_URL, { type: "module" });
  worker.addEventListener("message", handleWorkerMessage);
  worker.addEventListener("error", ({ message }) => finishWithError(message));
  worker.postMessage({ type: "source", name: activeSource.name, bytes: activeSource.bytes });
  return worker;
}

function clearTranscript() {
  elements.transcript.replaceChildren();
  const empty = document.createElement("p");
  empty.className = "playground__empty";
  empty.textContent = "Terminal cleared.";
  elements.transcript.append(empty);
}

function startTranscriptEntry(command) {
  const empty = elements.transcript.querySelector(".playground__empty");
  if (empty) empty.remove();
  const entry = document.createElement("article");
  entry.className = "playground__entry";
  entry.dataset.status = "running";

  const header = document.createElement("header");
  const prompt = document.createElement("code");
  prompt.textContent = `$ ${command}`;
  const status = document.createElement("span");
  status.className = "playground__duration";
  status.textContent = "Running…";
  header.append(prompt, status);

  const stdout = document.createElement("pre");
  stdout.className = "playground__stdout";
  const stderr = document.createElement("pre");
  stderr.className = "playground__stderr";
  stderr.hidden = true;
  entry.append(header, stdout, stderr);
  elements.transcript.prepend(entry);

  while (elements.transcript.querySelectorAll(".playground__entry").length > HISTORY_LIMIT) {
    elements.transcript.querySelector(".playground__entry:last-of-type").remove();
  }
  elements.transcript.scrollTop = 0;
  return entry;
}

function finishTranscriptEntry(entry, result, duration) {
  entry.dataset.status = result.status === 0 ? "ok" : "error";
  entry.dataset.exitStatus = String(result.status);
  const durationBadge = entry.querySelector(".playground__duration");
  const outcome = result.status === 0 ? "Succeeded" : "Failed";
  durationBadge.textContent = formatDuration(duration);
  durationBadge.title = outcome;
  durationBadge.setAttribute("aria-label", `${outcome} in ${formatDuration(duration)}`);
  entry.querySelector(".playground__stdout").textContent = result.stdout || "";
  const stderr = entry.querySelector(".playground__stderr");
  stderr.textContent = result.stderr || "";
  stderr.hidden = !result.stderr;
  elements.transcript.scrollTop = 0;
}

function finishWithError(message) {
  if (!runningEntry) return;
  finishTranscriptEntry(
    runningEntry,
    { stdout: "", stderr: `fatal: browser: ${message}\n`, status: 1 },
    performance.now() - runningStarted,
  );
  runningEntry = undefined;
  stopWorker();
}

function handleWorkerMessage({ data }) {
  if (data.id !== runningId || !runningEntry) return;
  if (data.type === "result") {
    finishTranscriptEntry(runningEntry, data.result, performance.now() - runningStarted);
    runningEntry = undefined;
    setRunning(false);
  } else if (data.type === "error") {
    finishWithError(data.message);
  }
}

function runCommand() {
  if (runningEntry) return;
  if (!activeSource) {
    elements["command-error"].textContent = "Choose a waveform first";
    return;
  }
  let argv;
  try {
    argv = currentTokens();
    elements["command-error"].textContent = "";
  } catch (error) {
    elements["command-error"].textContent = error.message;
    return;
  }

  runningId += 1;
  runningCommand = renderCommand(argv);
  if (commandHistory.at(-1) !== elements["command-line"].value) {
    commandHistory.push(elements["command-line"].value);
    commandHistory = commandHistory.slice(-HISTORY_LIMIT);
  }
  historyIndex = null;
  runningStarted = performance.now();
  runningEntry = startTranscriptEntry(runningCommand);
  setRunning(true);
  ensureWorker().postMessage({ type: "run", id: runningId, argv });
}

function stopCommand() {
  if (!worker || !runningEntry) return;
  runningId += 1;
  finishTranscriptEntry(
    runningEntry,
    { stdout: "", stderr: "Command stopped. Run again to start a fresh worker.\n", status: 1 },
    performance.now() - runningStarted,
  );
  runningEntry = undefined;
  stopWorker();
}

function navigateHistory(direction) {
  if (!commandHistory.length) return;
  if (historyIndex === null) {
    historyDraft = elements["command-line"].value;
    historyIndex = commandHistory.length;
  }
  historyIndex = Math.max(0, Math.min(commandHistory.length, historyIndex + direction));
  elements["command-line"].value = historyIndex === commandHistory.length
    ? historyDraft
    : commandHistory[historyIndex];
  synchronizeOutputMode();
}

for (const button of document.querySelectorAll("[data-example]")) {
  button.addEventListener("click", () => installCommandHelp(button.dataset.example));
}
for (const button of document.querySelectorAll("[data-suggestion]")) {
  button.addEventListener("click", () => installExample(button.dataset.suggestion));
}
for (const input of outputModes) input.addEventListener("change", () => selectOutput(input.value));
elements["command-line"].addEventListener("input", synchronizeOutputMode);
elements["command-line"].addEventListener("keydown", (event) => {
  if (event.key === "Enter") {
    event.preventDefault();
    runCommand();
  } else if (event.key === "ArrowUp" || event.key === "ArrowDown") {
    event.preventDefault();
    navigateHistory(event.key === "ArrowUp" ? -1 : 1);
  } else if (event.ctrlKey && event.key.toLowerCase() === "k") {
    event.preventDefault();
    clearTranscript();
  }
});
elements.run.addEventListener("click", () => {
  if (runningEntry) stopCommand();
  else runCommand();
});
elements.clear.addEventListener("click", clearTranscript);
elements["use-demo"].addEventListener("click", useDemo);
elements["open-local"].addEventListener("click", () => elements["local-file"].click());
function showCopyStatus(message) {
  elements["copy-status"].textContent = message;
  elements["copy-agent-prompt"].textContent = message;
  setTimeout(() => {
    if (elements["copy-status"].textContent === message) {
      elements["copy-status"].textContent = "";
      elements["copy-agent-prompt"].textContent = "Copy";
    }
  }, 2000);
}

elements["copy-agent-prompt"].addEventListener("click", async () => {
  try {
    await navigator.clipboard.writeText(elements["agent-prompt"].dataset.copy);
    showCopyStatus("Copied");
  } catch {
    const range = document.createRange();
    range.selectNodeContents(elements["agent-prompt"]);
    window.getSelection().removeAllRanges();
    window.getSelection().addRange(range);
    showCopyStatus("Press Ctrl+C");
  }
});
elements["local-file"].addEventListener("change", async ({ target }) => {
  const file = target.files[0];
  if (!file) return;
  if (!/\.(vcd|fst)$/i.test(file.name)) {
    elements["source-status"].textContent = "Choose a .vcd or .fst file";
    elements["source-indicator"].dataset.status = "error";
    return;
  }
  const generation = ++sourceGeneration;
  elements["source-name"].textContent = file.name;
  elements["source-size"].textContent = formatBytes(file.size);
  elements["source-format"].textContent = file.name.split(".").pop().toUpperCase();
  elements["source-status"].textContent = "Loading…";
  elements["source-indicator"].dataset.status = "loading";
  const bytes = await file.arrayBuffer();
  if (generation === sourceGeneration) setSource(file.name, bytes, "local");
  target.value = "";
});

const surfer = new URL("https://app.surfer-project.org/");
surfer.searchParams.set("load_url", DEMO_URL.href);
elements["open-surfer"].href = surfer.href;
elements["command-line"].value = examples.info;
synchronizeOutputMode();
useDemo();
