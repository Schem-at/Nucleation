// Deterministic fixture generator for the door_6x6 certificate.
// Run: npm run fixtures  (writes mock/fixtures/certificate.json)
//
// Story encoded in the data: 90 sim ticks. Lever flipped ON at tick 10 —
// redstone burst, then pistons walk the door open over 14 ticks (done at 24).
// Lever flipped OFF at tick 60 — second burst, door closes over 16 ticks
// (done at 76). Everything else is quiet.
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));

// Tiny deterministic PRNG so reruns are identical.
let s = 12345;
const rnd = () => ((s = (s * 1103515245 + 12345) & 0x7fffffff) / 0x7fffffff);

const SIM_TICKS = 90;
const OPEN_AT = 10; // lever on
const CLOSE_AT = 60; // lever off
const OPEN_TICKS = 14;
const CLOSE_TICKS = 16;

const events = [];
for (let t = 0; t < SIM_TICKS; t++) {
  let piston = 0;
  let redstone = 0;
  const burst = (start, len) => {
    if (t < start || t > start + len) return;
    const p = (t - start) / len;
    // redstone leads (wiring settles first), pistons follow in waves
    redstone += Math.max(0, Math.round(7 * Math.exp(-4 * p) + rnd() * 1.5 - 0.5));
    if (t >= start + 2) {
      const q = (t - start - 2) / len;
      piston += Math.max(0, Math.round(5 * Math.sin(Math.PI * Math.min(1, q)) * (0.7 + rnd() * 0.6)));
    }
  };
  burst(OPEN_AT, OPEN_TICKS);
  burst(CLOSE_AT, CLOSE_TICKS);
  const changes = piston * 2 + Math.round(redstone * 0.6);
  events.push({ tick: t, piston, redstone, changes });
}

// Heatmap: 22 wide x 13 tall footprint. 6x6 door leaf centered-ish
// (x 8..13, y 3..8) churns the most; piston columns flank it; redstone
// wiring trails toward the lever at [10,4].
const W = 22;
const H = 13;
const values = [];
for (let y = 0; y < H; y++) {
  const row = [];
  for (let x = 0; x < W; x++) {
    let v = 0;
    const inDoor = x >= 8 && x <= 13 && y >= 3 && y <= 8;
    const pistonCol = (x === 6 || x === 7 || x === 14 || x === 15) && y >= 3 && y <= 8;
    const wiring = y <= 2 && x >= 4 && x <= 17;
    const frame = (x === 5 || x === 16 || y === 9) && x >= 5 && x <= 16;
    if (inDoor) v = 8 + Math.round(rnd() * 6); // moving leaf blocks
    else if (pistonCol) v = 4 + Math.round(rnd() * 4); // piston extend/retract
    else if (wiring) v = rnd() < 0.55 ? 1 + Math.round(rnd() * 3) : 0; // dust flicker
    else if (frame) v = rnd() < 0.3 ? 1 : 0;
    row.push(v);
  }
  values.push(row);
}

const materials = [
  { id: "minecraft:white_concrete", count: 54 },
  { id: "minecraft:redstone_wire", count: 41 },
  { id: "minecraft:smooth_stone", count: 38 },
  { id: "minecraft:sticky_piston", count: 24 },
  { id: "minecraft:redstone_torch", count: 17 },
  { id: "minecraft:repeater", count: 14 },
  { id: "minecraft:observer", count: 12 },
  { id: "minecraft:slime_block", count: 8 },
  { id: "minecraft:redstone_block", count: 6 },
  { id: "minecraft:comparator", count: 4 },
  { id: "minecraft:noteblock", count: 2 },
  { id: "minecraft:lever", count: 1 },
].sort((a, b) => b.count - a.count);

const cert = {
  name: "door_6x6",
  dims: [22, 13, 7],
  lever: [10, 4, 1],
  open_ticks: OPEN_TICKS,
  close_ticks: CLOSE_TICKS,
  materials,
  events_per_tick: events,
  heatmap: { w: W, h: H, values },
  animation_url: "/api/doors/abc123/animation.mp4",
  sim_ticks: SIM_TICKS,
  seed: 12345,
};

fs.writeFileSync(path.join(here, "certificate.json"), JSON.stringify(cert, null, 1));
console.log("wrote certificate.json");
