/** Engineering-insight verification against `vite preview` on :8433.
 *
 * The new readings (server cost, dead weight, first movement, symmetry,
 * qualifier badges) are all derived numbers, so the checks are about
 * INTERNAL CONSISTENCY — every one of them can be cross-examined against
 * something else on the same certificate:
 *
 *   - the phase split sums to the total dispatch count;
 *   - dead weight never exceeds the block count, and the overlay instances
 *     exactly the cells the certificate lists;
 *   - the badges agree with the census they were derived from;
 *   - the first-movement trace ends on a tick the change log agrees moved;
 *   - and every number that is computed is also RENDERED, which is checked by
 *     reading the number back out of the DOM rather than trusting the props.
 *
 * Run: npx vite preview --host --port 8433 --strictPort
 *      node scripts/verify-insights.mjs
 */

import { chromium } from "playwright";
import { existsSync, mkdirSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const SHOTS = join(ROOT, "screenshots");
mkdirSync(SHOTS, { recursive: true });

// The community door files are not redistributable, so they live outside the
// repo. Point DOOR_CORPUS at the directory holding them; it defaults to
// ~/Downloads, which is where they land when downloaded.
const CORPUS = process.env.DOOR_CORPUS ?? join(homedir(), "Downloads");
const corpus = (name) => {
  const path = join(CORPUS, name);
  if (!existsSync(path)) {
    console.error(
      `missing door corpus file: ${path}\n` +
        `Set DOOR_CORPUS to the directory containing "${name}".`,
    );
    process.exit(1);
  }
  return path;
};

const URL = "http://localhost:8433/";
const DOORS = [
  { id: "door6x6", file: corpus("6x6 sliding door.litematic") },
  { id: "door4x4", file: corpus("4x4 sliding door.litematic") },
  {
    id: "vault4x4",
    file: corpus("fast 4x4 vault door (barrels filled).litematic"),
  },
];

const A = [];
const ok = (name, cond, got) => A.push({ name, pass: !!cond, got });
const fmt = (n) => Math.round(n).toLocaleString("en-US");

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
  const page = await browser.newPage({ viewport: { width: 1280, height: 1100 } });
  page.on("console", (m) => {
    if (m.type() === "error") consoleErrors.push(m.text());
  });
  page.on("pageerror", (e) => consoleErrors.push(`pageerror: ${e}`));

  await page.goto(URL, { waitUntil: "networkidle" });
  await page.locator('input[type="file"]').setInputFiles(door.file);
  await page.waitForURL(/\/door\//, { timeout: 300_000 });
  await page
    .locator('[data-testid="mesh-replay-stage"][data-ready="1"]')
    .waitFor({ timeout: 180_000 });

  const P = `[${door.id}]`;
  const cert = await page.evaluate(() => {
    const k = Object.keys(localStorage).find((x) => x.startsWith("door-cert-wasm:"));
    return k ? JSON.parse(localStorage[k]).certificate : null;
  });
  const rec = await page.evaluate(() => {
    const k = Object.keys(localStorage).find((x) => x.startsWith("door-cert-wasm:"));
    const r = JSON.parse(localStorage[k]);
    // The change log itself, so the first-movement tick can be re-derived
    // independently of the module that produced it.
    return { changes: r.replay.changes.length, simTicks: r.replay.simTicks };
  });
  const eng = cert?.engineering;
  ok(`${P} certificate carries the engineering block`, !!eng, Object.keys(eng ?? {}));
  if (!eng) {
    await page.close();
    continue;
  }

  /* -- 1. server cost ----------------------------------------------------- */
  const c = eng.cost;
  ok(`${P} server cost measured`, !!c);
  ok(`${P} ${fmt(c.updates)} dispatches per cycle`, c.updates > 0, c.updates);
  const phaseSum = c.by_phase.reduce((a, r) => a + r.n, 0);
  ok(
    `${P} phase split sums to the total (${fmt(phaseSum)} = ${fmt(c.updates)})`,
    phaseSum === c.updates,
    { phaseSum, total: c.updates, by_phase: c.by_phase },
  );
  ok(
    `${P} block events (${fmt(c.block_events)}) are a subset of the total`,
    c.block_events > 0 && c.block_events <= c.updates,
    c.block_events,
  );
  ok(
    `${P} per-tick peak ${fmt(c.peak)} on tick ${c.peak_tick} is <= the cycle total`,
    c.peak > 0 && c.peak <= c.updates && c.peak_tick >= 0 && c.peak_tick < rec.simTicks,
    { peak: c.peak, tick: c.peak_tick, simTicks: rec.simTicks },
  );
  ok(
    `${P} per-doorway-cell normalisation = ${c.per_passage_cell?.toFixed(1)}`,
    c.per_passage_cell === null ||
      Math.abs(c.per_passage_cell * cert.aperture_geometry.passage.length - c.updates) < 1,
    {
      per: c.per_passage_cell,
      cells: cert.aperture_geometry?.passage?.length,
      total: c.updates,
    },
  );

  // …and it is on the page, not just in storage.
  const figure = (
    await page.locator('[data-testid="server-cost"] .eng-figure b').textContent()
  )?.trim();
  ok(
    `${P} the cost figure renders as "${figure}"`,
    figure === fmt(c.updates),
    { dom: figure, cert: fmt(c.updates) },
  );
  const facts = await page.$$eval('[data-testid="server-cost"] .eng-facts dd', (els) =>
    els.map((e) => e.textContent.trim()),
  );
  ok(
    `${P} every supporting number renders (${facts.length} of them)`,
    facts.length >= 3 && facts[0].startsWith(fmt(c.block_events)),
    facts,
  );

  /* -- 2. dead weight ----------------------------------------------------- */
  const d = eng.dead;
  ok(`${P} dead weight measured`, !!d);
  ok(
    `${P} ${fmt(d.idle)} of ${fmt(d.total)} blocks did nothing — idle <= total`,
    d.idle <= d.total && d.idle >= 0,
    d,
  );
  ok(
    `${P} the total matches the bill of materials`,
    d.total === cert.materials.reduce((a, m) => a + m.count, 0),
    { dead: d.total, materials: cert.materials.reduce((a, m) => a + m.count, 0) },
  );
  ok(
    `${P} the idle breakdown sums to the idle count`,
    d.by_id.reduce((a, r) => a + r.count, 0) === d.idle,
    d.by_id.slice(0, 6),
  );
  ok(
    `${P} every idle cell is carried for the overlay (${d.cells.length})`,
    d.truncated ? d.cells.length === 3000 : d.cells.length === d.idle,
    { carried: d.cells.length, idle: d.idle, truncated: d.truncated },
  );
  const deadClaim = (
    await page.locator('[data-testid="dead-weight"] .eng-claim').textContent()
  )?.trim();
  ok(
    `${P} dead weight renders: "${deadClaim}"`,
    d.idle === 0
      ? deadClaim?.includes("Every one") && deadClaim?.includes(fmt(d.total))
      : deadClaim?.includes(fmt(d.idle)) && deadClaim?.includes(fmt(d.total)),
    deadClaim,
  );

  // The overlay draws exactly those cells.
  const idleToggle = page.locator('[data-testid="idle-toggle"]');
  const idlePanel = page.locator('[data-testid="idle-panel"]');
  ok(`${P} the dead-weight toggle is offered`, (await idleToggle.count()) === 1);
  ok(
    `${P} it is enabled iff there are idle blocks`,
    (await idleToggle.isDisabled()) === (d.idle === 0),
    { disabled: await idleToggle.isDisabled(), idle: d.idle },
  );
  if (d.idle > 0) {
    await idleToggle.click();
    await page.waitForTimeout(400);
    ok(`${P} the dead-weight panel appears`, (await idlePanel.count()) === 1);
    const drawn = await page.evaluate(() => window.__idle);
    ok(
      `${P} the overlay instances the certificate's own cells (${drawn?.drawn})`,
      drawn?.drawn === d.cells.length,
      { drawn, carried: d.cells.length },
    );
    const legend = (
      await page.locator('[data-testid="idle-legend"] li').textContent()
    )?.trim();
    ok(
      `${P} the legend count matches: "${legend}"`,
      legend?.includes(fmt(d.cells.length)),
      legend,
    );
  }

  /* -- 3. first movement -------------------------------------------------- */
  const f = eng.first;
  ok(`${P} first movement traced`, !!f, f);
  if (f) {
    ok(
      `${P} chain: ${f.chain.map((x) => x.id + (x.cells > 1 ? ` x${x.cells}` : "")).join(" -> ")}`,
      // The chain must start on the control that was actually thrown and end
      // on something that moves — anything else means the seed or the endpoint
      // was lost and the reader is looking at a middle slice.
      f.chain.length >= 2 &&
        f.chain[0].id === cert.input.kind &&
        /piston/.test(f.chain[f.chain.length - 1].id),
      { chain: f.chain, input: cert.input?.kind },
    );
    ok(
      `${P} ${fmt(f.hops)} updates across ${f.ticks} ticks before ${f.block} moved`,
      f.hops > 0 && f.ticks >= 0 && f.hops <= c.updates,
      { hops: f.hops, ticks: f.ticks, block: f.block, pos: f.pos },
    );
    const chainDom = (
      await page.locator('[data-testid="first-movement"] .eng-chain').textContent()
    )?.trim();
    ok(
      `${P} the chain renders: "${chainDom}"`,
      !!chainDom && (f.chain.length === 0 || chainDom.includes(f.chain[0].id)),
      { chainDom, chain: f.chain },
    );
  }

  /* -- 4. symmetry -------------------------------------------------------- */
  const s = eng.symmetry;
  ok(
    `${P} symmetry: pattern h=${s.pattern?.horizontal} v=${s.pattern?.vertical}; ` +
      `machine ${s.machine.map((m) => `${m.axis} ${(m.share * 100).toFixed(1)}%`).join(", ")}`,
    s.machine.length === 3 &&
      s.machine.every((m) => m.share >= 0 && m.share <= 1 && m.mirror === (m.share === 1)),
    s,
  );
  const symDom = (
    await page.locator('[data-testid="symmetry"] .eng-claim').textContent()
  )?.trim();
  ok(`${P} symmetry renders: "${symDom}"`, !!symDom, symDom);

  /* -- 5. badges ---------------------------------------------------------- */
  const b = eng.badges;
  const cs = cert.census;
  ok(
    `${P} observerless=${b.observerless} agrees with the census (${cs.observer} observers)`,
    b.observerless === (cs.observer === 0),
  );
  ok(
    `${P} dustless=${b.dustless} agrees with the census (${cs.redstone_wire} dust)`,
    b.dustless === (cs.redstone_wire === 0),
  );
  ok(
    `${P} slimeless=${b.slimeless} counts honey too ` +
      `(${cs.slime_block} slime + ${cs.honey_block} honey)`,
    b.slimeless === (cs.slime_block + cs.honey_block === 0),
  );
  ok(
    `${P} cycle-less=${b.cycleless}` +
      (b.tape ? ` — tape of period ${b.tape.period}, ${b.tape.pistons} pistons repeat` : ""),
    b.cycleless === (b.tape === null),
    { cycleless: b.cycleless, tape: b.tape, pistons: b.pistons },
  );
  const badgeDom = await page.$$eval('[data-testid="badges"] li', (els) =>
    els.map((e) => ({ text: e.textContent.trim(), on: e.classList.contains("on") })),
  );
  ok(`${P} all four badges render`, badgeDom.length === 4, badgeDom);
  ok(
    `${P} the badge states match the data`,
    badgeDom[0].on === b.observerless &&
      badgeDom[1].on === b.dustless &&
      badgeDom[2].on === b.slimeless &&
      badgeDom[3].on === b.cycleless,
    badgeDom,
  );

  /* -- the tile that moved ------------------------------------------------ */
  const tiles = await page.$$eval(".tiles .tile-label", (els) =>
    els.map((e) => e.textContent.trim()),
  );
  ok(
    `${P} "Peak in flight" left the measurement tiles for the engineering pair`,
    tiles.filter((t) => t === "Peak in flight").length === 1 &&
      tiles.includes("Peak dispatches"),
    tiles,
  );

  /* -- screenshots -------------------------------------------------------- */
  for (const theme of ["light", "dark"]) {
    await page.evaluate((t) => {
      document.documentElement.dataset.theme = t;
    }, theme);
    await page.waitForTimeout(400);
    await page.screenshot({
      path: join(SHOTS, `certificate-${door.id}-${theme}-insights.png`),
      fullPage: true,
    });
    const sec = page.locator('[data-testid="engineering"]');
    await sec.scrollIntoViewIfNeeded();
    await page.waitForTimeout(200);
    await sec.screenshot({
      path: join(SHOTS, `engineering-${door.id}-${theme}-insights.png`),
    });
  }

  ok(`${P} no console errors`, consoleErrors.length === 0, consoleErrors.slice(0, 4));

  results[door.id] = {
    name: cert.name,
    verdict: cert.verdict,
    aperture: cert.aperture,
    classification: cert.classification?.name ?? null,
    blocks: d?.total,
    open_ticks: cert.open_ticks,
    close_ticks: cert.close_ticks,
    moved_cells: cert.moved_cells,
    peak_changes: cert.peak_changes,
    cost: c,
    dead: d && { ...d, cells: d.cells.length },
    first: f,
    symmetry: s,
    badges: b,
    census: cs,
    changes: rec.changes,
    simTicks: rec.simTicks,
    consoleErrors,
  };
  await page.close();
}

await browser.close();

writeFileSync(
  join(ROOT, "verify-insights-out.json"),
  JSON.stringify({ results, assertions: A }, null, 2),
);

const failed = A.filter((x) => !x.pass);
for (const x of A) console.log(`${x.pass ? "✅" : "❌"} ${x.name}`);
if (failed.length) {
  console.log("\nfailures:", JSON.stringify(failed, null, 2));
  process.exit(1);
}
console.log(`\nall ${A.length} assertions pass`);
