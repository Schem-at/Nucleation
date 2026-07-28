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

  // Two marks on a door with no carriers, three on one that has them: the
  // pattern blocks and the carriers behind them are named separately.
  const wantMarks = facts.carrierCells > 0 ? 3 : 2;
  ok(`${P} legend names all ${wantMarks} marks`, legend.length === wantMarks, legend);
  ok(
    `${P} legend counts are the drawn cells (${legend.join(" | ")})`,
    legend[0]?.includes(facts.passageCells.toLocaleString("en-US")) &&
      legend[1]?.includes(facts.visibleCells.toLocaleString("en-US")) &&
      (wantMarks === 2 ||
        legend[2]?.includes(facts.carrierCells.toLocaleString("en-US"))),
    { legend, facts },
  );
  ok(
    `${P} pattern + carriers account for every door block ` +
      `(${facts.visibleCells} + ${facts.carrierCells} = ${facts.closedCells})`,
    facts.visibleCells + facts.carrierCells === facts.closedCells,
    facts,
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
  const N = (v) => v.toLocaleString("en-US");
  ok(
    `${P} summary line reads "${summary}"`,
    summary === `${facts.w} × ${facts.h} opening · ` +
      `${N(facts.passageCells)} passage cells · ` +
      (facts.carrierCells > 0
        ? `${N(facts.visibleCells)} pattern + ${N(facts.carrierCells)} carrier blocks`
        : `${N(facts.closedCells)} door blocks`) +
      ` · ${facts.depth} deep`,
    summary,
  );

  /* -- the half-block check: overlay vs meshed block ---------------------- */
  //
  // The bug this catches is a rendering one, so it is measured rather than
  // eyeballed: take a cell that holds a door block, and compare where the
  // MESHER put that block with where the overlay drew the same cell. A block
  // occupies the unit cube from its own corner, so both boxes must be
  // [p, p+1] on every axis, and their centres must coincide.
  const { align, statics } = await page.evaluate(() => {
    const k = Object.keys(localStorage).find((x) => x.startsWith("door-cert-wasm:"));
    const geo = JSON.parse(localStorage[k]).certificate.aperture_geometry;
    return {
      align: geo.closed.map((p) => window.__align(p)).filter(Boolean),
      statics: window.__alignStatic(40),
    };
  });
  const near = (v, want) => Math.abs(v - want) < 0.02;

  ok(`${P} probed ${align.length} door cells`, align.length > 0, align.length);
  // The group's own position is the convention, and it holds for every cell
  // whether the block is parked or being carried.
  ok(
    `${P} every group sits at its cell CENTRE — the mesher's GLB is ` +
      `origin-centred, so this is what puts the block in [p, p+1]`,
    align.every(
      (a) => a.moving || (a.groupPos && a.groupPos.every((v, i) => near(v, a.cell[i] + 0.5))),
    ),
    align.find((a) => !a.moving && !a.groupPos?.every((v, i) => near(v, a.cell[i] + 0.5))),
  );
  ok(
    `${P} the overlay marks every door cell at that same centre`,
    align.every(
      (a) => a.overlayCentre && a.overlayCentre.every((v, i) => near(v, a.cell[i] + 0.5)),
    ),
    align.find((a) => !a.overlayCentre?.every((v, i) => near(v, a.cell[i] + 0.5))) ?? align[0],
  );

  // Blocks that never move pin the box exactly: a vanilla model lives inside
  // its own cell, and a full cube fills it.
  const inCell = (a) =>
    a.meshMin.every((v, i) => v >= a.cell[i] - 0.02) &&
    a.meshMax.every((v, i) => v <= a.cell[i] + 1.02);
  ok(`${P} found ${statics.length} static blocks to measure`, statics.length > 0);
  ok(
    `${P} every static block's mesh stays inside its own cell [p, p+1]`,
    statics.length > 0 && statics.every(inCell),
    statics.find((a) => !inCell(a)),
  );
  const full = statics.find(
    (a) =>
      a.meshMin.every((v, i) => near(v, a.cell[i])) &&
      a.meshMax.every((v, i) => near(v, a.cell[i] + 1)),
  );
  ok(
    full
      ? `${P} full cube ${full.state} at (${full.cell}) meshes to exactly ` +
          `[${full.meshMin.map((v) => v.toFixed(3))}] → ` +
          `[${full.meshMax.map((v) => v.toFixed(3))}]`
      : `${P} a full-cube block meshes to exactly [p, p+1]`,
    !!full,
    statics.slice(0, 3),
  );
  // The headline number: the meshed block and the overlay mark on the SAME
  // cell must share a centre. A 0.5 gap on any axis is the bug.
  const paired = full && align.find((a) => a.overlayCentre);
  ok(
    `${P} block centre and overlay centre coincide — no half-block offset`,
    !!full &&
      [0, 1, 2].every((i) => near((full.meshMin[i] + full.meshMax[i]) / 2, full.cell[i] + 0.5)) &&
      !!paired &&
      paired.overlayCentre.every((v, i) => near(v, paired.cell[i] + 0.5)),
    {
      block: full && {
        cell: full.cell,
        centre: full.meshMin?.map((v, i) => (v + full.meshMax[i]) / 2),
      },
      overlay: paired && { cell: paired.cell, centre: paired.overlayCentre },
    },
  );
  ok(
    `${P} floor grid lines land on block boundaries`,
    align[0]?.gridX?.length > 1 &&
      align[0].gridX.every((x) => Math.abs(x - Math.round(x)) < 1e-6),
    align[0]?.gridX?.slice(0, 6),
  );

  /* -- the pattern is the visible surface --------------------------------- */
  const cls = cert.classification;
  ok(`${P} classification reads both faces`, (cls?.surfaces?.length ?? 0) >= 1, {
    surfaces: cls?.surfaces?.map((s) => `${s.side}: ${s.pattern ?? "none"}`),
  });
  ok(
    `${P} pattern is "${cls?.pattern ?? "unclassified"}" ${cls?.patternRef ?? ""}`,
    true,
    {
      name: cls?.name,
      dual: cls?.dual,
      volumePattern: cls?.volumePattern,
      surfaces: cls?.surfaces?.map((s) => ({
        side: s.side,
        pattern: s.pattern,
        ref: s.patternRef,
        transform: s.transform,
        ascii: s.ascii,
      })),
    },
  );
  // Flush is a FRAME property (Defs 2.6-2.8), never the door's name.
  ok(
    `${P} the headline names the pattern, not the frame ("${cls?.name}")`,
    !/Flush|Deluxe|Trapdoor/.test(cls?.name ?? ""),
    cls?.name,
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
      path: join(SHOTS, `certificate-${door.id}-${theme}-doorway-visible.png`),
      fullPage: true,
    });
  }

  /* -- close-up of the hole itself ---------------------------------------- */
  // Two shots, both framed on the doorway: the overlay showing pattern panes
  // against carrier cores, and the same view with the overlay off so the
  // meshed blocks can be checked against the grid they now line up with.
  await page.evaluate((t) => {
    document.documentElement.dataset.theme = t;
  }, "dark");
  // Park on the frame where the door is SHUT. That is the only frame where the
  // pattern blocks are in the doorway at all, so it is the only one where
  // "pane on block" and "pattern vs carrier" are visible things to check.
  //
  // Tick 0 is NOT always it: a file saved with its doorway standing open
  // measures its own closing first, and this 4 × 4 vault is one of those.
  const shutTick = cert.rest_is_closed ? 0 : (cert.close_ticks ?? 0) + 2;
  await page.locator(".replay-track input[type='range']").fill(String(shutTick));
  await page.evaluate(() => window.__doorway.focus());
  await page.waitForTimeout(800);
  await stage.screenshot({
    path: join(SHOTS, `${door.id}-doorway-closeup-visible.png`),
  });
  await toggle.click();
  await page.waitForTimeout(400);
  await stage.screenshot({
    path: join(SHOTS, `${door.id}-alignment-closeup-visible.png`),
  });
  await toggle.click();
  await page.waitForTimeout(300);

  /* -- the classification block, as it now reads -------------------------- */
  const classifyEl = page.locator('[data-testid="classification"]');
  if (await classifyEl.count()) {
    await classifyEl.scrollIntoViewIfNeeded();
    await page.waitForTimeout(200);
    await classifyEl.screenshot({
      path: join(SHOTS, `${door.id}-classification-visible.png`),
    });
  }
  const headline = (
    await page.locator('[data-testid="classify-name"]').textContent()
  )?.trim();
  ok(`${P} classification headline reads "${headline}"`, !!headline, headline);

  ok(`${P} no console errors`, consoleErrors.length === 0, consoleErrors.slice(0, 4));

  results[door.id] = {
    aperture: cert?.aperture,
    facts,
    legend,
    summary,
    headline,
    align: align.slice(0, 6),
    classification: cls && {
      name: cls.name,
      pattern: cls.pattern,
      patternRef: cls.patternRef,
      qualifiers: cls.qualifiers,
      frameNote: cls.frameNote,
      dual: cls.dual,
      volumePattern: cls.volumePattern,
      surfaces: cls.surfaces?.map((s) => ({
        side: s.side,
        pattern: s.pattern,
        patternRef: s.patternRef,
        transform: s.transform,
        cells: s.cells,
        layers: s.layers,
        ascii: s.ascii,
      })),
    },
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
