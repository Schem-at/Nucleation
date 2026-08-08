/** Does draining cost more the longer the session runs?
 *
 * `drainChanges` used to call `changesJson()`, which serialises the whole
 * accumulated log, then throw away everything before a cursor. That is
 * O(total changes) per frame and unbounded over a session. After the fix it
 * consumes and clears, so a drain costs what the last batch produced.
 *
 * `http://localhost:8455/` is where `vite preview` serves the app's `dist/`
 * build — run that (or `npm run preview` in this app) before this script.
 *
 * `placeBlock(31, 7, 13, "minecraft:air")` is BB-specific: it is BB's kick
 * block, and this build does nothing at all until it is kicked. Pass a
 * different `<BB>` fixture and this probe will step 400 quiescent ticks and
 * report meaningless, near-zero numbers for both samples.
 */
import { chromium } from "playwright";
const b = await chromium.launch(); const p = await b.newPage();
await p.goto("http://localhost:8455/", { waitUntil: "networkidle" });
await p.setInputFiles("input[type=file]", process.argv[2]);
await p.waitForSelector(".status.ready", { timeout: 300000 });
console.log(await p.evaluate(async () => {
  const w = window.simlab.world, out = [];
  w.sim.placeBlock(31, 7, 13, "minecraft:air"); // BB's kick block; see header
  const sample = () => {
    const t0 = performance.now();
    const got = w.drainChanges().length;
    return { ms: performance.now() - t0, got };
  };
  let first = null, last = null;
  for (let t = 0; t < 400; t++) {
    w.sim.step();
    const s = sample();
    if (t === 20) first = s;
    if (t === 399) last = s;
  }
  out.push("-- phase 1: no recording, clears every drain --");
  out.push(`drain at tick  20: ${first.ms.toFixed(3)} ms for ${first.got} changes`);
  out.push(`drain at tick 400: ${last.ms.toFixed(3)} ms for ${last.got} changes`);
  const grew = last.ms > first.ms * 4 && last.ms > 0.5;
  out.push(grew
    ? `  ❌ draining got ${(last.ms / Math.max(first.ms, 0.001)).toFixed(1)}x more expensive as the log grew`
    : "  ✅ a drain costs what the last batch produced, not what the session accumulated");

  // -- phase 2: a run timeline is recording, so every clearChanges() is
  // refused and the log is never emptied. `drainChanges` used to fall back
  // to `changesJson()` — the whole recorded-so-far log, every call — plus a
  // slice, which reserialised the entire backlog on every drain and made
  // cost climb with how long the recording had been running. It now asks
  // the engine for `changesJsonFrom(drainCursor)`, so a drain only pays for
  // the tail since the last one, exactly as phase 1's clearing path does.
  //
  // Two independent checks, because one proves cost and the other proves
  // correctness and neither implies the other: a drain could stay cheap by
  // silently dropping changes, or stay correct by paying an unbounded cost
  // to get there.
  //
  // Timing: same shape as phase 1's gate, restored now that reading from a
  // cursor makes it meaningful again — it did not hold against the old
  // whole-log reparse, which is why it was replaced with conservation-only
  // in that version.
  //
  // Conservation: every change the log ever held during the recording is
  // delivered to a drain exactly once — the sum of every batch this loop
  // receives must equal the final cumulative count. If the cursor were
  // broken (stuck at 0, or not advancing on a refusal, or advancing by the
  // wrong amount now that each read is a tail rather than the whole log),
  // that sum would run ahead of the final count instead, because most of it
  // would be the same entries returned again and again.
  w.startRecording();
  let sumGot = 0;
  let rFirst = null, rLast = null;
  for (let t = 0; t < 200; t++) {
    w.sim.step();
    const s = sample();
    sumGot += s.got;
    if (t === 10) rFirst = s;
    if (t === 199) rLast = s;
  }
  const finalTotal = Number(w.sim.changesCount());
  w.stopRecording();
  out.push("-- phase 2: recording, clears refused, cursor fallback --");
  out.push(`drain at tick  10 of recording: ${rFirst.ms.toFixed(3)} ms for ${rFirst.got} changes`);
  out.push(`drain at tick 200 of recording: ${rLast.ms.toFixed(3)} ms for ${rLast.got} changes`);
  const recGrew = rLast.ms > rFirst.ms * 4 && rLast.ms > 0.5;
  out.push(recGrew
    ? `  ❌ draining while recording got ${(rLast.ms / Math.max(rFirst.ms, 0.001)).toFixed(1)}x more expensive as the log grew`
    : "  ✅ a drain costs what the last batch produced, not what the recording has accumulated");
  out.push(`sum of every batch delivered over the recording: ${sumGot}; final cumulative log length: ${finalTotal}`);
  const conserved = sumGot === finalTotal;
  out.push(conserved
    ? "  ✅ every change was delivered exactly once — the cursor fallback tracks the refused clear correctly, no resend of the backlog"
    : `  ❌ delivered total (${sumGot}) diverges from the log (${finalTotal}) — a change was dropped or re-sent`);

  return out.join("\n");
}));
await b.close();
