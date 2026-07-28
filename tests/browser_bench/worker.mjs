// Module Web Worker running TickSimulation — each worker instantiates its own
// wasm module, which is how a browser GA parallelizes across cores. The import
// is dynamic so a load failure reports instead of hanging the page.
console.log("worker: booted");

const RB = "minecraft:redstone_block";

self.onmessage = async ({ data: { snbt, evals } }) => {
  try {
    const { TickSimulation, TickSettleMode } = await import(
      "/dist/npm-mctick/index.mjs"
    );
    console.log("worker: wasm module loaded");
    const evalOnce = () => {
      const sim = TickSimulation.fromSnbt(snbt, TickSettleMode.Quiet, 0, 0, 0, "");
      for (let t = 0; t < 80; t++) {
        if (t === 2) sim.placeBlock(2, 1, 1, RB);
        if (t === 4) sim.placeBlock(2, 1, 1, "minecraft:air");
        sim.step();
      }
      return sim.nonAirMinX() - 1;
    };
    for (let i = 0; i < 10; i++) evalOnce();
    const t0 = performance.now();
    let displacement = 0;
    for (let i = 0; i < evals; i++) displacement = evalOnce();
    self.postMessage({
      evalsPerSec: (1000 * evals) / (performance.now() - t0),
      displacement,
    });
  } catch (e) {
    self.postMessage({ error: String(e && e.stack ? e.stack : e) });
  }
};
