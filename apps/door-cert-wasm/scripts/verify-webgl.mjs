/** WebGL mesh-replay verification: uploads the 4x4 sliding door corpus snbt
 * against the vite preview on :8433, waits for the real-mesher WebGL stage,
 * captures at-rest and mid-stroke screenshots (`-webgl` suffix), reads mesh
 * timing + frame-time stats, and screenshots the iso fallback for contrast.
 *
 * Run: node scripts/verify-webgl.mjs
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
  const t = m.text();
  if (m.type() === "error" || m.type() === "warning") out.console.push(`${m.type()}: ${t}`);
  if (t.startsWith("[mesh]")) out.results.meshLog = t;
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

// Let it play ~6s so frame stats accumulate, then read p50/p95.
await page.waitForTimeout(6000);
out.results.frameStats = await page.evaluate(() => window.__replayFrameStats ?? null);

// Helper: seek the (uncontrolled) range imperatively through React's onChange.
const seek = async (t) => {
  await page.evaluate((v) => {
    const el = document.querySelector('.replay-track input[type="range"]');
    const set = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value").set;
    set.call(el, String(v));
    el.dispatchEvent(new Event("input", { bubbles: true }));
  }, t);
  await page.waitForTimeout(400); // a few rAFs to settle the pose
};

// Measured flip ticks from the scrubber marks (title = "label · t=N").
const marks = await page.locator(".replay-mark").evaluateAll((els) =>
  els.map((e) => Number((e.title.match(/t=(\d+)/) ?? [])[1])).filter((n) => !isNaN(n)),
);
out.results.flipTicks = marks;

// At rest (t=0).
await seek(0);
await stage.screenshot({ path: join(SHOTS, "door-replay-rest-webgl.png") });

// Mid-stroke: shortly after the first measured flip the pistons are moving.
const flip = marks.length ? marks[Math.floor(marks.length / 2)] : 10;
await seek(flip + 2.5);
await stage.screenshot({ path: join(SHOTS, "door-replay-midstroke-webgl.png") });
out.results.midStrokeTick = flip + 2.5;
await seek(flip + 4.5);
await stage.screenshot({ path: join(SHOTS, "door-replay-midstroke2-webgl.png") });

// Full page for context.
await page.screenshot({ path: join(SHOTS, "door-cert-page-webgl.png"), fullPage: true });

// Iso fallback still works behind the toggle.
await page.getByTestId("replay-mode-toggle").click();
await page.waitForTimeout(600);
await page
  .locator(".replay-stage svg")
  .screenshot({ path: join(SHOTS, "door-replay-iso-fallback.png") });

await browser.close();
console.log(JSON.stringify(out, null, 2));
