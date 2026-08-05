/** Drive the observer/diagonal-piston counter through the real page.
 *
 * Clicks the note block N times and reports, after each, where the concrete
 * sits and whether the lower piston moved — the machine's whole point. Run
 * against `npx vite preview --port 8455`.
 */
import { chromium } from "playwright";

const file = process.argv[2];
const clicks = Number(process.argv[3] ?? 4);

const b = await chromium.launch();
const p = await b.newPage();
const errs = [];
p.on("pageerror", (e) => errs.push(String(e).slice(0, 200)));
p.on("console", (m) => { if (m.type() === "error") errs.push(m.text().slice(0, 200)); });

await p.goto("http://127.0.0.1:8455/", { waitUntil: "networkidle" });
await p.setInputFiles("input[type=file]", file);
await p.waitForSelector(".status.ready, .status.error", { timeout: 300000 });
console.log("load:", (await p.textContent(".status")).trim());

const out = await p.evaluate(async (clicks) => {
  const w = window.simlab?.world;
  if (!w?.sim) return { error: "no sim" };
  const watch = ["-1,0,0", "0,0,0", "1,0,0", "2,1,0", "3,1,0"];
  const read = () =>
    Object.fromEntries(
      watch.map((k) => {
        const [x, y, z] = k.split(",").map(Number);
        return [k, w.blockAt(x, y, z).replace("minecraft:", "")];
      }),
    );
  const log = [{ when: "settled", ...read() }];
  for (let c = 1; c <= clicks; c++) {
    w.sim.useBlock(6, 1, 0);
    for (let i = 0; i < 12; i++) w.sim.step();
    w.applyChanges(w.drainChanges());
    log.push({ when: `click ${c}`, ...read() });
  }
  // does the renderer know about anything at negative x?
  const chunks = [...w.group.children].length;
  return { log, chunks, settle: w.settle };
}, clicks);

if (out.error) console.log("ERROR:", out.error);
else {
  console.log("settle mode:", out.settle, " chunks in scene:", out.chunks);
  console.table(out.log);
}
console.log("errors:", errs.length ? errs.slice(0, 3) : "none");
await b.close();
