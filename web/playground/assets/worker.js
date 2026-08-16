let wasm;
let source;

async function loadWasm() {
  if (!wasm) {
    wasm = import("./wasm/wavepeek.js").then(async (module) => {
      await module.default();
      return module;
    });
  }
  return wasm;
}

self.addEventListener("message", async ({ data }) => {
  if (data.type === "source") {
    source = { name: data.name, bytes: data.bytes };
    self.postMessage({ type: "source-ready" });
    return;
  }
  if (data.type !== "run") return;

  try {
    if (!source) throw new Error("No waveform is loaded");
    const module = await loadWasm();
    const result = JSON.parse(
      module.run_wavepeek(JSON.stringify(data.argv), source.name, source.bytes),
    );
    self.postMessage({ type: "result", id: data.id, result });
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    self.postMessage({ type: "error", id: data.id, message });
  }
});
