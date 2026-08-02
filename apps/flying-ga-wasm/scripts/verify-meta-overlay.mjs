/** Verification for the 3-D meta-structure overlay on the machine viewer.
 *
 * Evolves a short GA run so a genuinely evolved machine (kicker + dead weight)
 * exists, then splices two hand-built reference machines into the stored run's
 * leaderboard — the canonical 6x1x1 flying machine from
 * crates/mc-tick/tests/corpus/structures/flying_machine.snbt, and that same
 * machine with cargo bolted on — and shoots the viewer for each, in both
 * themes and with each overlay layer toggled.
 *
 * Run: node scripts/verify-meta-overlay.mjs [port] [runSeconds]
 */

import { chromium } from "playwright";
import { mkdirSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const SHOTS = join(ROOT, "screenshots");
mkdirSync(SHOTS, { recursive: true });

const PORT = Number(process.argv[2] ?? 8477);
const RUN_SECONDS = Number(process.argv[3] ?? 55);
const URL = `http://localhost:${PORT}/`;

const out = { console: [], errors: [], results: {} };
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1480, height: 1180 } });
page.on("console", (m) => {
  if (m.type() === "error" || m.type() === "warning")
    out.console.push(`${m.type()}: ${m.text()}`);
});
page.on("pageerror", (e) => out.errors.push(String(e)));

await page.goto(URL, { waitUntil: "networkidle" });

/* ---------------------------------------------------- evolve something --- */

// bbox x must clear 6 for the canonical 6x1x1 to fit; engine-b seeding starts
// from a real flying machine so accreted payload/dead weight shows up inside a
// run short enough to survive the localStorage quota.
// The Genome group (bbox + gen-0 seed) is collapsed by default.
await page
  .locator(".cfg summary", {
    has: page.locator(".g-title", { hasText: /^Genome$/ }),
  })
  .click();
await page.getByLabel("Bounding box x").fill("8");
await page.getByTestId("cfg-seeding").selectOption("engine-b");
await page.locator("#cfg-pop").fill("48");
// 220 generations blew the localStorage quota and left an index entry with no
// record at all — saveRun keeps one filmstrip loop per champion generation.
await page.locator("#cfg-gens").fill("40");
// Reward inert mass and raise the block ceiling, or the run just polishes
// six-block bars and the overlay is never tested on anything dense.
await page.getByTestId("obj-cargo").check();
await page.getByTestId("cfg-maxb").fill("26");
// A floor on block count as well as a ceiling: left to itself the GA polishes
// six-block bars, and an overlay that has only ever been looked at on a bar has
// not been looked at.
await page.getByTestId("cfg-minb").fill("12");
// Ban note blocks: TickSimulation.fromSnbt throws NucleationError.Simulation on
// any structure containing one, so every machine that evolves one has NO static
// graph at all and the overlay has nothing to draw. Engine bug, reported — this
// keeps the visual verification on machines the analysis can actually see.
await page
  .locator(".chips button", { hasText: /^note block$/ })
  .first()
  .click();
out.results.cargoObjective = await page.getByTestId("obj-cargo").isChecked();
await page.getByTestId("start-run").click();
await page
  .getByTestId("stat-status")
  .filter({ hasText: "running" })
  .waitFor({ timeout: 90_000 });
console.log(`evolving (cap 60 gens, ${RUN_SECONDS}s budget) …`);
const t0 = Date.now();
while (Date.now() - t0 < RUN_SECONDS * 1000) {
  await page.waitForTimeout(3000);
  // NB: stat-status innerText is "Status\ndone" — test the whole node, not
  // its first line, or the done-check silently never fires.
  const status = await page.getByTestId("stat-status").innerText();
  const gen = (await page.getByTestId("stat-generation").innerText()).split("\n")[0];
  const best = (await page.getByTestId("stat-best").innerText()).split("\n")[0];
  console.log(`  t+${Math.round((Date.now() - t0) / 1000)}s gen=${gen} best=${best}`);
  if (/done/i.test(status)) break;
}
if (!/done/i.test(await page.getByTestId("stat-status").innerText()))
  await page.getByTestId("stop-run").click({ timeout: 15_000 });
await page
  .getByTestId("stat-status")
  .filter({ hasText: "done" })
  .waitFor({ timeout: 120_000 });
await page.waitForTimeout(1500);

/* -------------------------------- splice in the two reference machines --- */

// Reload FIRST. A finished run is still mounted in React and the app re-saves
// its in-memory record, silently reverting any edit made before this point —
// which is how an earlier version of this script shot engine-b and reported it
// as the canonical machine.
await page.reload({ waitUntil: "networkidle" });
await page.waitForTimeout(800);

out.results.splice = await page.evaluate(() => {
  // Genome state indices, in ALPHABET order (src/ga/alphabet.ts): air, slime,
  // honey, sticky_piston x6 facings, piston x6, observer x6, note_block,
  // white_concrete, oak_trapdoor x4.
  const F = ["east", "west", "north", "south", "up", "down"];
  const A = ["minecraft:air", "minecraft:slime_block", "minecraft:honey_block"];
  for (const f of F) A.push(`minecraft:sticky_piston[extended=false,facing=${f}]`);
  for (const f of F) A.push(`minecraft:piston[extended=false,facing=${f}]`);
  for (const f of F) A.push(`minecraft:observer[facing=${f},powered=false]`);
  A.push("minecraft:note_block", "minecraft:white_concrete");
  for (const f of F.slice(0, 4))
    A.push(
      `minecraft:oak_trapdoor[facing=${f},half=bottom,open=true,powered=false,waterlogged=false]`,
    );
  const idx = (s) => A.indexOf(s);

  // A long run can blow the quota and leave an index entry with no record, so
  // trust the keys rather than the index.
  const key = Object.keys(localStorage).find((k) => k.startsWith("fgaw:run:"));
  if (!key)
    return { ok: false, why: "no stored run record", keys: Object.keys(localStorage) };
  const rec = JSON.parse(localStorage.getItem(key) ?? "null");
  if (!rec) return { ok: false, why: "run record unparseable" };

  const bbox = rec.config.bbox;
  const [bx, , bz] = bbox;
  const cell = (x, y, z) => (y * bz + z) * bx + x;
  const blank = () => new Array(bbox[0] * bbox[1] * bbox[2]).fill(0);
  const blocksOf = (g) => {
    const outB = [];
    for (let y = 0; y < bbox[1]; y++)
      for (let z = 0; z < bbox[2]; z++)
        for (let x = 0; x < bbox[0]; x++) {
          const s = g[cell(x, y, z)];
          if (s !== 0) outB.push({ x, y, z, state: A[s] });
        }
    return outB;
  };

  // flying_machine.snbt, 6x1x1: observer(w) slime piston(e) piston(w) slime
  // observer(e). Engine should be all six cells, payload empty.
  const canonical = blank();
  const LINE = [
    idx("minecraft:observer[facing=west,powered=false]"),
    idx("minecraft:slime_block"),
    idx("minecraft:sticky_piston[extended=false,facing=east]"),
    idx("minecraft:sticky_piston[extended=false,facing=west]"),
    idx("minecraft:slime_block"),
    idx("minecraft:observer[facing=east,powered=false]"),
  ];
  LINE.forEach((s, x) => (canonical[cell(x, 0, 0)] = s));

  // Same machine with cargo bolted on: the six engine cells must stay the
  // engine and the concrete must land in payload, not in it.
  const cargo = canonical.slice();
  if (bbox[2] > 1) {
    cargo[cell(1, 0, 1)] = idx("minecraft:white_concrete");
    cargo[cell(2, 0, 1)] = idx("minecraft:white_concrete");
  }
  if (bbox[1] > 1) {
    cargo[cell(4, 1, 0)] = idx("minecraft:white_concrete");
  }

  const mk = (id, name, g) => ({
    id,
    name,
    fitness: 9.9,
    gen: 0,
    genome: g,
    blocks: blocksOf(g),
    speed: 0.99,
  });

  // Biggest first, so row 2 is the most interesting evolved machine.
  const evolved = [...(rec.leaderboard ?? [])].sort(
    (a, b) => b.blocks.length - a.blocks.length,
  );
  rec.leaderboard = [
    mk("ref-canonical", "canonical 6x1x1", canonical),
    mk("ref-cargo", "canonical + cargo", cargo),
    ...evolved.slice(0, 8),
  ];
  localStorage.setItem(key, JSON.stringify(rec));
  return {
    ok: true,
    runId: key,
    bbox,
    canonicalBlocks: blocksOf(canonical).length,
    cargoBlocks: blocksOf(cargo).length,
    evolved: rec.leaderboard.slice(2).map((m) => ({
      id: m.id,
      n: m.blocks.length,
    })),
  };
});
console.log("splice:", JSON.stringify(out.results.splice));
if (!out.results.splice.ok) {
  console.log(JSON.stringify(out, null, 2));
  await browser.close();
  process.exit(1);
}

/* ------------------------------------------------------------- shooting --- */

await page.reload({ waitUntil: "networkidle" });
// The sidebar stacks over the history list, so dispatch rather than click.
await page.locator(".hist-open").first().dispatchEvent("click");
await page.waitForTimeout(800);
// The run-config drawer sits over the whole page at this width; its backdrop
// swallows every later click on the overlay toggles.
const backdrop = page.locator(".drawer-backdrop");
if (await backdrop.count()) {
  await backdrop.dispatchEvent("click");
  await page.waitForTimeout(500);
}

const rows = page.locator(".lb-table tbody tr.row");
await rows.first().waitFor({ timeout: 30_000 });

/** Read the overlay legend counts so the picture can be checked against text. */
async function legend() {
  const has = await page.getByTestId("meta-legend").count();
  if (!has) return null;
  return page.evaluate(() => {
    const el = document.querySelector('[data-testid="meta-legend"]');
    if (!el) return null;
    const grab = (sel) =>
      [...el.querySelectorAll(sel + " li")].map((li) =>
        li.textContent.replace(/\s+/g, " ").trim(),
      );
    return {
      roles: grab(".meta-roles-legend"),
      graph: grab(".meta-graph-legend"),
      phase: el.querySelector(".meta-phase-note")?.textContent.slice(0, 60),
    };
  });
}

async function shoot(name) {
  await page.waitForTimeout(1400);
  const panel = page.locator("section.panel", { has: page.getByTestId("meta-controls") });
  await panel.screenshot({ path: join(SHOTS, `meta-${name}.png`) });
  return legend();
}

/** Select by NAME, never by row index: the row order is the app's business and
 * an index silently shoots whatever happens to be there. */
async function selectByName(name) {
  const row = page.locator(".lb-table tbody tr.row", {
    has: page.locator("td.name", { hasText: name }),
  });
  await row.first().dispatchEvent("click");
  await page.waitForTimeout(2400);
  const got = await page.getByTestId("viewer-machine").innerText();
  if (!got.includes(name))
    throw new Error(`selected "${got}" but wanted "${name}"`);
  return got;
}

out.results.shots = {};

// 1 — canonical 6x1x1.
out.results.shots.canonical = {
  machine: await selectByName("canonical 6x1x1"),
  legend: await shoot("canonical-light"),
};

// 2 — canonical + cargo.
out.results.shots.cargo = {
  machine: await selectByName("canonical + cargo"),
  legend: await shoot("cargo-light"),
};

// 3 — an evolved machine that actually exercises all four roles. Chosen by
// walking the leaderboard and reading the overlay's own legend, so the shot is
// of a machine that demonstrably has a kicker AND dead weight rather than one
// picked by rank and hoped over.
const names = await page.locator(".lb-table tbody tr.row td.name").allInnerTexts();
const num = (rows, k) => {
  const hit = (rows ?? []).find((r) => r.startsWith(k));
  return hit ? Number(hit.slice(k.length)) : 0;
};
let pick = null;
out.results.scan = [];
for (const name of names) {
  if (name.startsWith("canonical")) continue;
  await selectByName(name);
  const lg = await legend();
  const n = {
    name,
    engine: num(lg?.roles, "engine"),
    payload: num(lg?.roles, "payload"),
    kicker: num(lg?.roles, "kicker"),
    dead: num(lg?.roles, "dead weight"),
  };
  n.cells = n.engine + n.payload + n.kicker + n.dead;
  out.results.scan.push(n);
  if (n.kicker > 0 && n.dead > 0 && n.cells >= 10) {
    pick = name;
    break;
  }
}
if (!pick) {
  // Fall back to the richest machine on offer, and SAY it fell back.
  const best = [...out.results.scan].sort(
    (a, b) => b.kicker + b.dead + b.cells / 100 - (a.kicker + a.dead + a.cells / 100),
  )[0];
  pick = best?.name ?? names.find((n) => !n.startsWith("canonical"));
  out.results.evolvedFellBack = true;
}
out.results.evolvedPick = pick;
out.results.shots.evolved = {
  machine: await selectByName(pick),
  legend: await shoot("evolved-light"),
};

await page.getByTestId("meta-graph").uncheck();
await shoot("evolved-roles-only");
await page.getByTestId("meta-graph").check();
await page.getByTestId("meta-roles").uncheck();
await shoot("evolved-graph-only");
await page.getByTestId("meta-roles").check();
await page.getByTestId("meta-ghost").uncheck();
await shoot("evolved-no-xray");
await page.getByTestId("meta-ghost").check();

// Dark mode — the palette is selected per surface, so it must be shot per
// surface too.
await page.emulateMedia({ colorScheme: "dark" });
await page.waitForTimeout(1600);
out.results.shots.evolvedDark = { legend: await shoot("evolved-dark") };
await selectByName("canonical 6x1x1");
await shoot("canonical-dark");
await page.emulateMedia({ colorScheme: "light" });
await selectByName(pick);
await page.waitForTimeout(1500);
// Whole page: the flat per-y-layer panel below must still agree with the
// overlay above, cage for cage.
await page.screenshot({ path: join(SHOTS, "meta-page-light.png"), fullPage: true });

console.log(JSON.stringify(out, null, 2));
await browser.close();

let fail = 0;
const assert = (ok, msg) => {
  console.log(`${ok ? "PASS" : "FAIL"}: ${msg}`);
  if (!ok) fail = 1;
};
const can = out.results.shots.canonical.legend;
assert(!!can, "overlay legend rendered for the canonical machine");
assert(
  (can?.roles ?? []).some((r) => /^engine\s*6$/.test(r)),
  `canonical engine is all six cells (got ${JSON.stringify(can?.roles)})`,
);
assert(
  (can?.roles ?? []).some((r) => /^payload\s*0$/.test(r)),
  "canonical payload is empty",
);
const car = out.results.shots.cargo.legend;
assert(
  (car?.roles ?? []).some((r) => /^engine\s*6$/.test(r)),
  `cargo variant keeps the same six engine cells (got ${JSON.stringify(car?.roles)})`,
);
assert(
  (car?.roles ?? []).some((r) => /^payload\s*[1-9]/.test(r)),
  "cargo becomes payload",
);
const evo = out.results.shots.evolved.legend;
assert(
  (evo?.graph ?? []).some((g) => /^pushes\s*[1-9]/.test(g)),
  `evolved machine draws push edges (got ${JSON.stringify(evo?.graph)})`,
);
assert(
  (evo?.graph ?? []).some((g) => /^sticks to\s*[1-9]/.test(g)),
  "evolved machine draws sticks-to edges",
);
const ev = out.results.scan.find((s) => s.name === out.results.evolvedPick);
assert(
  (ev?.cells ?? 0) >= 10,
  `evolved machine is dense enough to be a real test (${ev?.cells} cells)`,
);
assert(
  (ev?.kicker ?? 0) > 0 && (ev?.dead ?? 0) > 0,
  `evolved machine exercises kicker AND dead weight (${JSON.stringify(ev)})`,
);
assert(out.errors.length === 0, `no page errors (${out.errors.join(" | ")})`);
process.exit(fail);
