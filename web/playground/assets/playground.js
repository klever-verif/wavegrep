const DEMO_NAME = "scr1_axi.fst";
const DEMO_URL = new URL("./scr1_axi.fst", import.meta.url);
const WORKER_URL = new URL("./worker.js", import.meta.url);
const HISTORY_LIMIT = 20;

const examples = [
  ["Metadata", `wavepeek info --waves ${DEMO_NAME}`],
  ["Hierarchy", `wavepeek scope --waves ${DEMO_NAME} --tree --max-depth 3 --max 80`],
  ["AXI signals", `wavepeek signal --waves ${DEMO_NAME} --scope TOP.scr1_top_tb_axi.i_top --filter '.*io_axi_dmem_(arvalid|arready|araddr).*' --max 40`],
  ["Point values", `wavepeek value --waves ${DEMO_NAME} --scope TOP.scr1_top_tb_axi.i_top --signals clk,io_axi_dmem_arvalid,io_axi_dmem_arready --at 1000ps`],
  ["Clocked changes", `wavepeek change --waves ${DEMO_NAME} --scope TOP.scr1_top_tb_axi.i_top --signals io_axi_dmem_arvalid,io_axi_dmem_arready --on 'posedge clk' --from 1ps --to 1880182ps --max 10`],
  ["Property matches", `wavepeek property --waves ${DEMO_NAME} --scope TOP.scr1_top_tb_axi.i_top --on 'posedge clk' --eval 'io_axi_dmem_arvalid && io_axi_dmem_arready' --capture match --from 1ps --to 1880182ps --max 10`],
  ["Generic extraction", `wavepeek extract generic --waves ${DEMO_NAME} --scope TOP.scr1_top_tb_axi.i_top --on 'posedge clk' --when 'io_axi_dmem_arvalid && io_axi_dmem_arready' --payload io_axi_dmem_araddr,io_axi_dmem_arlen --from 1ps --to 1880182ps --max 10`],
  ["AXI extraction", `wavepeek extract axi --waves ${DEMO_NAME} --scope TOP.scr1_top_tb_axi.i_top --include '^io_axi_dmem_' --map aclk=clk --map aresetn=axi_rst_n --from 1ps --to 1880182ps --max 10`],
];

const elements = Object.fromEntries(
  [
    "source-status", "use-demo", "local-file", "open-surfer", "surfer-note",
    "example", "command-kind", "output-mode", "command-line", "command-error",
    "run", "stop", "exit-status", "stdout", "stderr", "history",
  ].map((id) => [id, document.getElementById(id)]),
);
const optionInputs = [...document.querySelectorAll("[data-option]")];

let activeSource;
let worker;
let runningId = 0;
let runningCommand = "";
let history = [];

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

function optionValue(tokens, option) {
  const index = tokens.indexOf(option);
  if (index >= 0) return tokens[index + 1] ?? "";
  const prefix = `${option}=`;
  return tokens.find((token) => token.startsWith(prefix))?.slice(prefix.length) ?? "";
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

function commandLocation(tokens) {
  const commands = new Set(["info", "scope", "signal", "value", "change", "property", "extract"]);
  const index = tokens.findIndex((token, position) => position > 0 && commands.has(token));
  if (index < 0) return { index: 1, length: 0, value: "" };
  const length = tokens[index] === "extract" && tokens[index + 1] ? 2 : 1;
  return { index, length, value: tokens.slice(index, index + length).join(" ") };
}

function currentTokens() {
  const tokens = tokenize(elements["command-line"].value);
  if (!tokens.length) tokens.push("wavepeek");
  return tokens;
}

function writeTokens(tokens) {
  elements["command-line"].value = renderCommand(tokens);
  synchronizeControls();
}

function synchronizeControls() {
  try {
    const tokens = currentTokens();
    const command = commandLocation(tokens).value;
    if ([...elements["command-kind"].options].some((option) => option.value === command)) {
      elements["command-kind"].value = command;
    }
    elements["output-mode"].value = tokens.includes("--jsonl")
      ? "jsonl"
      : tokens.includes("--json") ? "json" : "human";
    for (const input of optionInputs) input.value = optionValue(tokens, input.dataset.option);
    elements["command-error"].textContent = "";
  } catch (error) {
    elements["command-error"].textContent = error.message;
  }
}

function updateControlledOption(input) {
  try {
    const tokens = currentTokens();
    setOption(tokens, input.dataset.option, input.value.trim());
    writeTokens(tokens);
  } catch (error) {
    elements["command-error"].textContent = error.message;
  }
}

function selectCommand(value) {
  const tokens = currentTokens();
  const current = commandLocation(tokens);
  tokens.splice(current.index, current.length, ...value.split(" "));
  writeTokens(tokens);
}

function selectOutput(value) {
  const tokens = currentTokens().filter((token) => token !== "--json" && token !== "--jsonl");
  if (value !== "human") tokens.push(`--${value}`);
  writeTokens(tokens);
}

function formatBytes(bytes) {
  return `${(bytes / 1024 / 1024).toFixed(2)} MiB`;
}

function stopWorker() {
  if (worker) worker.terminate();
  worker = undefined;
  elements.run.disabled = false;
  elements.stop.disabled = true;
}

function setSource(name, bytes, kind) {
  stopWorker();
  activeSource = { name, bytes: new Uint8Array(bytes), kind };
  elements["source-status"].textContent = `${name} · ${formatBytes(bytes.byteLength)} · ${kind === "demo" ? "bundled demo" : "local file"}`;
  elements["open-surfer"].hidden = kind !== "demo";
  elements["surfer-note"].textContent = kind === "demo"
    ? "Surfer opens the same public demo for visual exploration."
    : "Local files are never sent to Surfer. Open Surfer separately and load the file there yourself.";
  const tokens = currentTokens();
  setOption(tokens, "--waves", name);
  writeTokens(tokens);
}

async function useDemo() {
  elements["source-status"].textContent = "Loading bundled demo…";
  try {
    const response = await fetch(DEMO_URL);
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    setSource(DEMO_NAME, await response.arrayBuffer(), "demo");
  } catch (error) {
    elements["source-status"].textContent = `Could not load bundled demo: ${error.message}`;
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

function showResult(result) {
  elements.stdout.textContent = result.stdout || "";
  elements.stderr.textContent = result.stderr || "";
  elements["exit-status"].textContent = `Exit ${result.status}`;
  elements["exit-status"].dataset.status = result.status === 0 ? "ok" : "error";
}

function addHistory(command, result) {
  history.unshift({ command, result });
  history = history.slice(0, HISTORY_LIMIT);
  elements.history.replaceChildren(...history.map((entry) => {
    const item = document.createElement("li");
    const button = document.createElement("button");
    button.type = "button";
    button.textContent = `Exit ${entry.result.status} · ${entry.command}`;
    button.addEventListener("click", () => {
      elements["command-line"].value = entry.command;
      synchronizeControls();
      showResult(entry.result);
    });
    item.append(button);
    return item;
  }));
}

function finishWithError(message) {
  const result = { stdout: "", stderr: `fatal: browser: ${message}\n`, status: 1 };
  showResult(result);
  addHistory(runningCommand, result);
  stopWorker();
}

function handleWorkerMessage({ data }) {
  if (data.id !== runningId) return;
  if (data.type === "result") {
    showResult(data.result);
    addHistory(runningCommand, data.result);
    elements.run.disabled = false;
    elements.stop.disabled = true;
  } else if (data.type === "error") {
    finishWithError(data.message);
  }
}

function runCommand() {
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
  runningCommand = elements["command-line"].value;
  elements.run.disabled = true;
  elements.stop.disabled = false;
  elements["exit-status"].textContent = "Running…";
  ensureWorker().postMessage({ type: "run", id: runningId, argv });
}

function stopCommand() {
  if (!worker) return;
  runningId += 1;
  stopWorker();
  elements["exit-status"].textContent = "Stopped";
  elements.stderr.textContent = "Command stopped. Run again to start a fresh worker.\n";
}

for (const [label, command] of examples) {
  const option = document.createElement("option");
  option.textContent = label;
  option.value = command;
  elements.example.append(option);
}
elements.example.addEventListener("change", () => {
  elements["command-line"].value = elements.example.value;
  synchronizeControls();
});
elements["command-kind"].addEventListener("change", (event) => selectCommand(event.target.value));
elements["output-mode"].addEventListener("change", (event) => selectOutput(event.target.value));
for (const input of optionInputs) input.addEventListener("change", () => updateControlledOption(input));
elements["command-line"].addEventListener("input", synchronizeControls);
elements.run.addEventListener("click", runCommand);
elements.stop.addEventListener("click", stopCommand);
elements["use-demo"].addEventListener("click", useDemo);
elements["local-file"].addEventListener("change", async ({ target }) => {
  const file = target.files[0];
  if (!file) return;
  if (!/\.(vcd|fst)$/i.test(file.name)) {
    elements["source-status"].textContent = "Choose a .vcd or .fst file.";
    return;
  }
  setSource(file.name, await file.arrayBuffer(), "local");
  target.value = "";
});

const surfer = new URL("https://app.surfer-project.org/");
surfer.searchParams.set("load_url", DEMO_URL.href);
elements["open-surfer"].href = surfer.href;
elements["command-line"].value = examples[0][1];
synchronizeControls();
useDemo();
