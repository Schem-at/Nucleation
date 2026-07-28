/** Piston-arm clamp verification on the door replay: uploads the 4x4
 * sliding door corpus, pauses the WebGL mesh replay, zooms in, and captures
 * scrub-position close-ups through the extension and retraction windows
 * after each lever flip — the frames where the old fixed-length arm poked
 * out the back of its base. Also captures the iso (voxel) replay at the
 * same scrub positions (it uses the cast's clamped boxesForItem geometry).
 *
 * Run: node scripts/verify-arm-clamp.mjs   (vite preview on :8433)
 */

import { chromium } from "playwright";
import { mkdirSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const SHOTS = join(ROOT, "screenshots");
mkdirSync(SHOTS, { recursive: true });

const SNBT = join(
  ROOT,
  "../../crates/mc-tick/tests/corpus/structures/door_4x4_sliding.snbt",
);
const URL = "http://localhost:8433/";

const out = { console: [], errors: [], results: {} };
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1280, height: 1000 } });
page.on("console", (m) => {
  if (m.type() === "error") out.console.push(`${m.type()}: ${m.text()}`);
});
page.on("pageerror", (e) => out.errors.push(String(e)));

await page.goto(URL, { waitUntil: "networkidle" });
await page.locator('input[type="file"]').setInputFiles(SNBT);
await page.waitForURL(/\/door\//, { timeout: 120_000 });

const stage = page.getByTestId("mesh-replay-stage");
await stage.waitFor({ timeout: 60_000 });
await page
  .locator('[data-testid="mesh-replay-stage"][data-ready="1"]')
  .waitFor({ timeout: 120_000 });
await page.waitForTimeout(1000);

// Pause playback.
await page.getByRole("button", { name: "Pause replay" }).click();

// Seek helper (uncontrolled range, poked through the input event).
const seek = async (t) => {
  await page.evaluate((v) => {
    const el = document.querySelector('.replay-track input[type="range"]');
    const set = Object.getOwnPropertyDescriptor(
      HTMLInputElement.prototype,
      "value",
    ).set;
    set.call(el, String(v));
    el.dispatchEvent(new Event("input", { bubbles: true }));
  }, t);
  await page.waitForTimeout(350);
};

// Lever flip ticks from the scrubber marks (title = "label · t=N").
const marks = await page.locator(".replay-mark").evaluateAll((els) =>
  els.map((el) => Number((el.getAttribute("title") ?? "").match(/t=(\d+)/)?.[1])),
);
out.results.flipTicks = marks;

// Exact piston-head move windows from the cast (window.__doorCast).
const heads = await page.evaluate(() => {
  const cast = window.__doorCast ?? [];
  return cast
    .filter((m) => m.headBase && m.motion)
    .map((m) => ({
      state: m.state,
      x: m.x,
      y: m.y,
      z: m.z,
      base: m.headBase,
      start: m.start,
      until: m.motion.until,
      end: m.end,
      // Retracting: the member sits AT its base cell (head slides home).
      retracting: m.x === m.headBase[0] && m.y === m.headBase[1] && m.z === m.headBase[2],
    }));
});
out.results.headWindows = heads;

// Zoom in for close-ups (OrbitControls dolly via wheel at stage center).
const bb = await stage.boundingBox();
await page.mouse.move(bb.x + bb.width / 2, bb.y + bb.height / 2);
await page.mouse.wheel(0, -600);
await page.waitForTimeout(500);

// Capture the historical worst cases: EARLY EXTENSION (head just leaving
// its base) and LATE RETRACTION (head almost seated) — the times where the
// fixed-length arm used to poke out the back of the base.
const ext = heads.filter((h) => !h.retracting);
const ret = heads.filter((h) => h.retracting);
const times = [];
for (const h of ext.slice(0, 2))
  times.push({ phase: "ext-early", t: h.start + 0.25 * (h.until - h.start) });
for (const h of ret.slice(0, 2)) {
  times.push({ phase: "ret-late", t: h.start + 0.9 * (h.until - h.start) });
  times.push({ phase: "ret-seated", t: (h.until + h.end) / 2 });
}
const captures = [];
for (const { phase, t } of times) {
  await seek(t);
  const name = `door-scrub-${phase}-t${t.toFixed(2)}-webgl.png`;
  await stage.screenshot({ path: join(SHOTS, name) });
  captures.push(`screenshots/${name}`);
}

// Iso (voxel) replay at the same positions — clamped boxesForItem path.
await page.getByTestId("replay-mode-toggle").click();
await page.waitForTimeout(600);
const isoStage = page.locator(".replay-stage").first();
for (const { phase, t } of times) {
  await seek(t);
  const name = `door-scrub-${phase}-t${t.toFixed(2)}-iso.png`;
  await isoStage.screenshot({ path: join(SHOTS, name) });
  captures.push(`screenshots/${name}`);
}
out.results.captures = captures;

console.log(JSON.stringify(out, null, 2));
await browser.close();
console.log("VERIFY OK");
