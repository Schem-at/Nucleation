/** Doorway overlay verification against `vite preview` on :8433.
 *
 * The overlay's whole claim is that you no longer have to trust the aperture
 * number — you can look at the cells it was counted from. So the checks are
 * about agreement and independence, not about the picture being pretty:
 *
 *   - the overlay toggles independently of the x-ray, in either order, and
 *     renders in both modes;
 *   - the legend's counts are the same cells the certificate's aperture line
 *     was measured from (`aperture.cells` === w × h of the drawn passage,
 *     `aperture.depth` === the drawn depth), so the two cannot disagree;
 *   - the passage is a CLEAN rectangular opening — `w × h × depth` cells, no
 *     ragged edge. A failure here is a finding about the extractor, not about
 *     the overlay, and the run says so;
 *   - the overlay is static geometry, so it must not cost frame time.
 *
 * Run: npx vite preview --host --port 8433 --strictPort
 *      node scripts/verify-doorway.mjs
 */

import { chromium } from "playwright";
import { mkdirSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const SHOTS = join(ROOT, "screenshots");
mkdirSync(SHOTS, { recursive: true });

const URL = "http://localhost:8433/";
/** Each door with the opening the standard says it has. */
const DOORS = [
  { id: "door6x6", file: "/tmp/door6x6.litematic", w: 6, h: 6 },
  {
    id: "door4x4",
    file: "/Users/harrison/Downloads/fast 4x4 vault door (barrels filled).litematic",
    w: 4,
    h: 4,
  },
];

const A = [];
const ok = (name, cond, got) => A.push({ name, pass: !!cond, got });

const browser = await chromium.launch({
  args: [
    "--use-angle=metal",
    "--enable-gpu",
    "--ignore-gpu-blocklist",
    "--enable-unsafe-swiftshader",
  ],
});

const results = {};

for (const door of DOORS) {
  const consoleErrors = [];
  const page = await browser.newPage({ viewport: { width: 1280, height: 1000 } });
  page.on("console", (m) => {
    if (m.type() === "error") consoleErrors.push(m.text());
  });
  page.on("pageerror", (e) => consoleErrors.push(`pageerror: ${e}`));

  await page.goto(URL, { waitUntil: "networkidle" });
  await page.locator('input[type="file"]').setInputFiles(door.file);
  await page.waitForURL(/\/door\//, { timeout: 300_000 });
  const stage = page.locator('[data-testid="mesh-replay-stage"]');
  await page
    .locator('[data-testid="mesh-replay-stage"][data-ready="1"]')
    .waitFor({ timeout: 180_000 });

  const P = `[${door.id}]`;

  /* -- the certificate's own aperture line -------------------------------- */
  const cert = await page.evaluate(() => {
    const k = Object.keys(localStorage).find((x) =>
      x.startsWith("door-cert-wasm:"),
    );
    return k ? JSON.parse(localStorage[k]).certificate : null;
  });
  ok(`${P} certificate carries the doorway geometry`, !!cert?.aperture_geometry, {
    passage: cert?.aperture_geometry?.passage?.length,
    closed: cert?.aperture_geometry?.closed?.length,
  });

  /* -- frame-time control, overlay off ------------------------------------ */
  const measure = async (ms = 3000) => {
    await page.evaluate(() => window.__replayFrameReset());
    await page.waitForTimeout(ms);
    return page.evaluate(() => window.__replayFrameStats);
  };
  const framesPlain = await measure();

  /* -- the toggle, on the plain replay ------------------------------------ */
  const toggle = page.locator('[data-testid="doorway-toggle"]');
  const panel = page.locator('[data-testid="doorway-panel"]');
  const xrayToggle = page.locator('[data-testid="xray-toggle"]');
  ok(`${P} doorway toggle is offered`, (await toggle.count()) === 1);
  ok(`${P} toggle is enabled`, !(await toggle.isDisabled()));
  ok(`${P} overlay starts off`, (await panel.count()) === 0);

  // Park the replay so both runs draw the same frame, and so the door is shut
  // (the door blocks are in the passage — the nesting the overlay is for).
  await page.locator('.replay-track input[type="range"]').fill("0");
  await page.waitForTimeout(250);

  await toggle.click();
  await page.waitForTimeout(400);
  ok(`${P} overlay turns on WITHOUT x-ray`, (await panel.count()) === 1);
  ok(
    `${P} x-ray is still off`,
    (await stage.getAttribute("data-xray")) === "0",
  );
  const plainShot = await stage.screenshot();

  /* -- counts: the legend vs the certificate ------------------------------ */
  const facts = await page.evaluate(() => {
    const d = window.__doorway;
    return d && { ...d, focus: undefined };
  });
  const legend = await page.$$eval('[data-testid="doorway-legend"] li', (els) =>
    els.map((e) => e.textContent.trim()),
  );
  const summary = (
    await page.locator('[data-testid="doorway-summary"]').textContent()
  )?.trim();

  ok(`${P} legend names both marks`, legend.length === 2, legend);
  ok(
    `${P} legend counts are the drawn cells (${legend.join(" | ")})`,
    legend[0]?.includes(facts.passageCells.toLocaleString("en-US")) &&
      legend[1]?.includes(facts.closedCells.toLocaleString("en-US")),
    { legend, facts },
  );
  ok(
    `${P} drawn cells are the certificate's own geometry`,
    facts.passageCells === cert.aperture_geometry.passage.length &&
      facts.closedCells === cert.aperture_geometry.closed.length,
    [facts.passageCells, cert.aperture_geometry.passage.length],
  );
  ok(
    `${P} overlay opening ${facts.w} × ${facts.h} matches the certificate ` +
      `aperture ${cert.aperture?.w} × ${cert.aperture?.h}`,
    facts.opening === cert.aperture?.cells,
    { overlay: facts.opening, cert: cert.aperture?.cells },
  );
  ok(
    `${P} overlay depth ${facts.depth} matches the certificate depth ` +
      `${cert.aperture?.depth}`,
    facts.depth === cert.aperture?.depth,
    [facts.depth, cert.aperture?.depth],
  );
  ok(
    `${P} the opening is the expected ${door.w} × ${door.h}`,
    Math.min(facts.w, facts.h) === Math.min(door.w, door.h) &&
      Math.max(facts.w, facts.h) === Math.max(door.w, door.h),
    [facts.w, facts.h],
  );
  // The acceptance criterion the brief names: a CLEAN hole. If this fails the
  // overlay is telling the truth about a ragged extraction — report, do not tune.
  ok(
    `${P} the passage is a clean rectangular hole ` +
      `(${facts.passageCells} cells = ${facts.w}×${facts.h}×${facts.depth})`,
    facts.rectangular,
    facts,
  );
  ok(
    `${P} summary line reads "${summary}"`,
    summary === `${facts.w} × ${facts.h} opening · ` +
      `${facts.passageCells.toLocaleString("en-US")} passage cells · ` +
      `${facts.closedCells.toLocaleString("en-US")} door blocks · ` +
      `${facts.depth} deep`,
    summary,
  );

  /* -- independence: x-ray on UNDER a live overlay ------------------------ */
  await xrayToggle.click();
  await page.waitForTimeout(500);
  ok(
    `${P} overlay survives x-ray ON`,
    (await panel.count()) === 1 &&
      (await stage.getAttribute("data-xray")) === "1",
  );
  const bothShot = await stage.screenshot();
  ok(
    `${P} the overlay repaints for the darkroom`,
    Buffer.compare(plainShot, bothShot) !== 0,
  );
  const framesBoth = await measure();

  await page.screenshot({
    path: join(SHOTS, `certificate-${door.id}-dark-doorway-xray.png`),
    fullPage: true,
  });

  // …and off again, overlay untouched.
  await xrayToggle.click();
  await page.waitForTimeout(400);
  ok(
    `${P} overlay survives x-ray OFF`,
    (await panel.count()) === 1 &&
      (await stage.getAttribute("data-xray")) === "0",
  );

  const framesOverlay = await measure();
  ok(
    `${P} the overlay is free (p50 ${framesOverlay?.p50?.toFixed(1)} ms vs ` +
      `${framesPlain?.p50?.toFixed(1)} ms plain)`,
    framesOverlay && framesPlain && framesOverlay.p50 <= framesPlain.p50 + 1.5,
    { plain: framesPlain, overlay: framesOverlay, both: framesBoth },
  );

  /* -- the overlay goes away ---------------------------------------------- */
  await toggle.click();
  await page.waitForTimeout(300);
  ok(`${P} overlay turns off again`, (await panel.count()) === 0);
  await toggle.click();
  await page.waitForTimeout(300);

  /* -- screenshots: overlay alone, both themes ---------------------------- */
  for (const theme of ["light", "dark"]) {
    await page.evaluate((t) => {
      document.documentElement.dataset.theme = t;
    }, theme);
    await page.waitForTimeout(400);
    await page.screenshot({
      path: join(SHOTS, `certificate-${door.id}-${theme}-doorway.png`),
      fullPage: true,
    });
  }

  /* -- close-up of the hole itself ---------------------------------------- */
  await page.evaluate((t) => {
    document.documentElement.dataset.theme = t;
  }, "dark");
  await page.evaluate(() => window.__doorway.focus());
  await page.waitForTimeout(600);
  await stage.screenshot({
    path: join(SHOTS, `${door.id}-doorway-closeup.png`),
  });

  ok(`${P} no console errors`, consoleErrors.length === 0, consoleErrors.slice(0, 4));

  results[door.id] = {
    aperture: cert?.aperture,
    facts,
    legend,
    summary,
    framesPlain,
    framesOverlay,
    framesBoth,
    consoleErrors,
  };
  await page.close();
}

await browser.close();

writeFileSync(
  join(ROOT, "verify-doorway-out.json"),
  JSON.stringify({ results, assertions: A }, null, 2),
);

const failed = A.filter((x) => !x.pass);
for (const x of A) console.log(`${x.pass ? "✅" : "❌"} ${x.name}`);
if (failed.length) {
  console.log("\nfailures:", JSON.stringify(failed, null, 2));
  process.exit(1);
}
console.log(`\nall ${A.length} assertions pass`);
