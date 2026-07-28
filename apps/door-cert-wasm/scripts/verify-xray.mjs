/** X-ray propagation view verification against `vite preview` on :8433.
 *
 * What is being proved:
 *   - the update stream is recorded and reaches the page, and the DRAWABLE
 *     views stay small (the raw log is 15.8 MB/cycle and is never fetched);
 *   - the x-ray toggle does not disturb the replay it sits on;
 *   - the busiest tick of the cycle (19,834 updates) renders at frame rate;
 *   - the sub-tick scrubber actually moves the wavefront — two positions in
 *     the same tick must not produce the same picture;
 *   - both colour channels render with their legend.
 *
 * Run: npx vite preview --host --port 8433 --strictPort
 *      node scripts/verify-xray.mjs
 */

import { chromium } from "playwright";
import { mkdirSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const SHOTS = join(ROOT, "screenshots");
mkdirSync(SHOTS, { recursive: true });

const URL = "http://localhost:8433/";
const DOOR = "/tmp/door6x6.litematic";
const MB = 1024 * 1024;

const A = [];
const ok = (name, cond, got) => A.push({ name, pass: !!cond, got });

const consoleErrors = [];
// Frame times only mean anything on a real GPU: headless Chromium falls back
// to SwiftShader, where a transparent scene is fill-rate bound in software and
// every mode looks equally slow. ANGLE/Metal gives the numbers a meaning.
const browser = await chromium.launch({
  args: [
    "--use-angle=metal",
    "--enable-gpu",
    "--ignore-gpu-blocklist",
    "--enable-unsafe-swiftshader",
  ],
});
const page = await browser.newPage({ viewport: { width: 1280, height: 1000 } });
page.on("console", (m) => {
  if (m.type() === "error") consoleErrors.push(m.text());
});
page.on("pageerror", (e) => consoleErrors.push(`pageerror: ${e}`));

await page.goto(URL, { waitUntil: "networkidle" });
await page.locator('input[type="file"]').setInputFiles(DOOR);
await page.waitForURL(/\/door\//, { timeout: 300_000 });
const stage = page.locator('[data-testid="mesh-replay-stage"]');
await page.locator('[data-testid="mesh-replay-stage"][data-ready="1"]').waitFor({
  timeout: 180_000,
});

/* -- payload sizes ------------------------------------------------------- */
const meta = await page.evaluate(() => window.__xray ?? null);
ok("update stream reached the page", meta !== null, meta && meta.totalUpdates);
ok(
  `heat payload < 2 MB (${meta ? (meta.heatBytes / MB).toFixed(2) : "?"} MB)`,
  meta && meta.heatBytes < 2 * MB,
  meta?.heatBytes,
);
ok(
  "four tick phases recorded",
  meta && meta.phases.length === 4,
  meta?.phases,
);
ok("two update kinds recorded", meta && meta.kinds.length === 2, meta?.kinds);

const busiest = meta
  ? meta.updatesPerTick.indexOf(Math.max(...meta.updatesPerTick))
  : 0;
const busiestN = meta ? meta.updatesPerTick[busiest] : 0;

/* -- frame-time control: the ordinary replay, same scene, same camera ----- */
const measure = async (ms = 3500) => {
  await page.evaluate(() => window.__replayFrameReset());
  await page.waitForTimeout(ms);
  return page.evaluate(() => window.__replayFrameStats);
};
const framesPlain = await measure();

/* -- toggling keeps playback state --------------------------------------- */
const toggle = page.locator('[data-testid="xray-toggle"]');
ok("x-ray toggle is offered", (await toggle.count()) === 1);
ok("toggle enabled once the stream is recorded", !(await toggle.isDisabled()));

// Park the replay somewhere identifiable, paused, then toggle twice.
await page.locator('.replay-track input[type="range"]').fill("12");
await page.waitForTimeout(200);
const tickBefore = (await page.locator(".replay-readout b").textContent())?.trim();
await toggle.click();
await page.waitForTimeout(400);
const xrayOnAttr = await stage.getAttribute("data-xray");
const tickDuringXray = (await page.locator(".replay-readout b").textContent())?.trim();
ok("x-ray turns on", xrayOnAttr === "1");
ok("panel appears", (await page.locator('[data-testid="xray-panel"]').count()) === 1);
ok(
  `tick survives x-ray on (${tickBefore} -> ${tickDuringXray})`,
  tickBefore === tickDuringXray,
  [tickBefore, tickDuringXray],
);
await toggle.click();
await page.waitForTimeout(300);
const tickAfter = (await page.locator(".replay-readout b").textContent())?.trim();
ok(
  `tick survives x-ray off (${tickAfter})`,
  tickBefore === tickAfter && (await stage.getAttribute("data-xray")) === "0",
  [tickBefore, tickAfter],
);
ok(
  "panel goes away with the mode",
  (await page.locator('[data-testid="xray-panel"]').count()) === 0,
);

/* -- both colour channels render with a legend --------------------------- */
await toggle.click();
await page.waitForTimeout(300);
const legendPhase = await page.$$eval('[data-testid="xray-legend"] li', (els) =>
  els.map((e) => e.textContent.trim()),
);
ok("phase legend lists four phases", legendPhase.length === 4, legendPhase);
const phaseShot = await stage.screenshot();

await page.locator('[data-testid="xray-channel-kind"]').click();
await page.waitForTimeout(500);
const legendKind = await page.$$eval('[data-testid="xray-legend"] li', (els) =>
  els.map((e) => e.textContent.trim()),
);
ok("kind legend lists two kinds", legendKind.length === 2, legendKind);
const kindShot = await stage.screenshot();
ok(
  "the two channels render differently",
  Buffer.compare(phaseShot, kindShot) !== 0,
  [phaseShot.length, kindShot.length],
);
await page.locator('[data-testid="xray-channel-phase"]').click();
await page.waitForTimeout(200);

/* -- sub-tick: the busiest tick, at frame rate --------------------------- */
await page.locator('[data-testid="xray-enter-busiest"]').click();
await page.waitForTimeout(400);
const readout = page.locator('[data-testid="xray-readout"] b');
const firstReadout = (await readout.textContent()) ?? "";
ok(
  `sub-tick readout names the sequence (${firstReadout})`,
  /update [\d,]+ \/ [\d,]+ · phase \w+ · \w+ · /.test(firstReadout),
  firstReadout,
);

// Measure a clean window while the sweep plays across the busiest tick.
const frames = await measure();
const flares = await page.evaluate(() => window.__xrayFlares);
ok(
  `busiest tick (${busiestN} updates) holds >= 30 fps ` +
    `(p50 ${frames?.p50?.toFixed(1)} ms, p95 ${frames?.p95?.toFixed(1)} ms; ` +
    `plain replay p50 ${framesPlain?.p50?.toFixed(1)} ms)`,
  frames && frames.p50 <= 33.4,
  { xray: frames, plain: framesPlain },
);
ok(
  "the x-ray costs no more than the replay it sits on",
  frames && framesPlain && frames.p50 <= framesPlain.p50 + 2,
  [frames?.p50, framesPlain?.p50],
);
ok("flares are actually drawn", flares > 0, flares);

/* -- the scrubber moves the wavefront ------------------------------------ */
const seq = page.locator('[data-testid="xray-seq"]');
const setSeq = async (v) => {
  await seq.evaluate((el, val) => {
    const setter = Object.getOwnPropertyDescriptor(
      window.HTMLInputElement.prototype,
      "value",
    ).set;
    setter.call(el, String(val));
    el.dispatchEvent(new Event("input", { bubbles: true }));
    el.dispatchEvent(new Event("change", { bubbles: true }));
  }, v);
  await page.waitForTimeout(350);
};

const a = Math.floor(busiestN * 0.12);
const b = Math.floor(busiestN * 0.62);
await setSeq(a);
const shotA = await stage.screenshot();
const textA = (await readout.textContent()) ?? "";
await setSeq(b);
const shotB = await stage.screenshot();
const textB = (await readout.textContent()) ?? "";
ok(
  `seq ${a} and seq ${b} render different wavefronts`,
  Buffer.compare(shotA, shotB) !== 0,
  [shotA.length, shotB.length],
);
// The readout counts updates from 1, the scrubber indexes from 0.
ok(
  "the readout follows the scrubber",
  textA !== textB && textA.startsWith(`update ${(a + 1).toLocaleString("en-US")} /`),
  [textA, textB],
);

// Precise stepping: one update at a time.
await page.locator('[data-testid="xray-step-fwd"]').click();
await page.waitForTimeout(250);
const textStep = (await readout.textContent()) ?? "";
ok(
  "single-update stepping advances by exactly one",
  textStep.startsWith(`update ${(b + 2).toLocaleString("en-US")} /`),
  [textB, textStep],
);

/* -- mid-wave frame + themed full pages ---------------------------------- */
await setSeq(Math.floor(busiestN * 0.38));
await stage.screenshot({ path: join(SHOTS, "xray-wave-midtick.png") });
const sb = await stage.boundingBox();
await page.screenshot({
  path: join(SHOTS, "xray-subtick-controls.png"),
  clip: { x: sb.x - 2, y: sb.y - 2, width: sb.width + 4, height: sb.height + 190 },
});

await page.locator('[data-testid="xray-exit-subtick"]').click();
await page.waitForTimeout(200);
await page.locator('.replay-track input[type="range"]').fill("6");
await page.waitForTimeout(400);
for (const theme of ["light", "dark"]) {
  await page.evaluate((t) => {
    document.documentElement.dataset.theme = t;
  }, theme);
  await page.waitForTimeout(350);
  await page.screenshot({
    path: join(SHOTS, `certificate-door6x6-${theme}-xray.png`),
    fullPage: true,
  });
}

ok("no console errors", consoleErrors.length === 0, consoleErrors.slice(0, 4));

const out = {
  door: "door6x6",
  payload: meta && {
    heatBytes: meta.heatBytes,
    heatMB: +(meta.heatBytes / MB).toFixed(3),
    waveBytes: meta.waveBytes,
    waveMB: +(meta.waveBytes / MB).toFixed(3),
    cells: meta.cells,
    ticks: meta.ticks,
    totalUpdates: meta.totalUpdates,
    heatRef: meta.heatRef,
    phases: meta.phases,
    kinds: meta.kinds,
    busiestTick: busiest,
    busiestUpdates: busiestN,
  },
  frames,
  framesPlain,
  flares,
  legendPhase,
  legendKind,
  consoleErrors,
  assertions: A,
};
writeFileSync(join(ROOT, "verify-xray-out.json"), JSON.stringify(out, null, 2));
await browser.close();

const failed = A.filter((x) => !x.pass);
for (const x of A) console.log(`${x.pass ? "✅" : "❌"} ${x.name}`);
if (failed.length) {
  console.log("\nfailures:", JSON.stringify(failed, null, 2));
  process.exit(1);
}
console.log(`\nall ${A.length} assertions pass`);
