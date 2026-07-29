/** Entity rendering on the mesh replay, end to end.
 *
 * Neither browser app drew entities: `TickSimulation.itemEntitiesJson()`
 * reports live minecarts and dropped items and nothing consumed it. This
 * checks the whole path — worker samples the tracks, the mesher encodes the
 * vanilla models, the replay animates them — and captures it.
 *
 * The fixture is built here rather than committed: the 4x4 sliding door from
 * the corpus, plus a stone floor and a rail lane down its free side, with a
 * minecart rolling along the rails and two dropped items beside it. Building
 * it from a corpus door means the page it renders on is a real certificate,
 * and the cart travels while the door strokes, which is the point — a static
 * entity would prove nothing about the replay timeline.
 *
 * The regression set is re-run in the same pass. Entity sampling costs one
 * JSON read on a build with no entities, and every door in the corpus has
 * none, so these numbers must not move:
 *
 *   6x6 sliding      36 cells, CERTIFIED
 *   4x4 sliding      16 cells, CERTIFIED
 *   4x4 vault (filled) 16 cells, CERTIFIED
 *   2x2 flush x2      4 cells
 *   0.45_4x4_funnel   loads
 *
 * Run: npm run build && npx vite preview --port 8477 --strictPort
 *      node scripts/verify-entities.mjs
 */
import { chromium } from "playwright";
import { mkdirSync, writeFileSync, readFileSync, existsSync } from "node:fs";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const SHOTS = join(ROOT, "screenshots");
mkdirSync(SHOTS, { recursive: true });
/** Fixtures are derived from the corpus on every run, so they are build
 *  output, not source: they go to the temp dir rather than into the repo.
 *  `TMPDIR` is not always a temp directory — some shells here export the repo
 *  root as `TMPDIR`, and a blind `join(tmpdir(), ...)` then drops generated
 *  files into the working tree. Only an absolute path outside `ROOT` is one. */
const TMP = (() => {
  const t = tmpdir();
  return t.startsWith("/") && !ROOT.startsWith(t) ? t : "/tmp";
})();
const WORK = join(TMP, "door-cert-entities");
mkdirSync(WORK, { recursive: true });
const HOME = process.env.HOME;
const URL = "http://localhost:8477/";

/* ------------------------------------------------------------- fixture --- */

/** A corpus door with a rail lane, a minecart and two dropped items.
 *
 * `gametestSnbt` emits `entities: []` for a schematic that carries none, which
 * is the injection point: the engine's SNBT loader reads the entities list and
 * spawns them, so a hand-written cart arrives in the simulation as a live one.
 * (Entities in a .litematic/.schem are still dropped on the way in — that is
 * the bridge's half of this work, and it is why the fixture goes through SNBT.)
 */
async function buildFixture() {
  const eng = await import(join(ROOT, "public/engine/index.mjs"));
  const src = `${HOME}/Downloads/4x4 sliding door.litematic`;
  if (!existsSync(src)) throw new Error(`corpus door missing: ${src}`);
  const schem = eng.Schematic.fromData(
    Array.from(new Uint8Array(readFileSync(src))),
  );
  // The lane goes on the door's free z=3 face, which is empty at ground level
  // bar one block at x=9. It has to stay INSIDE the file's own 16x9x4 bounds:
  // writing a block outside them does not grow the region by one, it blows the
  // reported dimensions out to nonsense (z=4 reports a depth of 69), and the
  // size guard would refuse the fixture before anything got drawn.
  const LANE_Z = 3;
  for (let x = 0; x <= 8; x++) schem.setBlockFromString(x, 0, LANE_Z, "minecraft:stone");
  for (let x = 0; x <= 5; x++)
    schem.setBlockFromString(x, 1, LANE_Z, "minecraft:rail[shape=east_west,waterlogged=false]");
  const entities = [
    // On the rails at one end, rolling east: ~0.18 blocks/tick, so it crosses
    // the lane over the door's stroke rather than teleporting or sitting still.
    `{pos:[0.5d,1.0d,${LANE_Z}.5d],blockPos:[0,1,${LANE_Z}],nbt:{id:"minecraft:minecart",` +
      `Pos:[0.5d,1.0d,${LANE_Z}.5d],Motion:[0.18d,0.0d,0.0d],Rotation:[0.0f,0.0f]}}`,
    // Dropped items resting on the stone floor: one gem (drawn as its block
    // form) and one shulker box (drawn as the real base + lid).
    `{pos:[7.5d,1.4d,${LANE_Z}.5d],blockPos:[7,1,${LANE_Z}],nbt:{id:"minecraft:item",` +
      `Pos:[7.5d,1.4d,${LANE_Z}.5d],Motion:[0.0d,0.0d,0.0d],Item:{id:"minecraft:diamond",Count:5b}}}`,
    `{pos:[8.5d,1.4d,${LANE_Z}.5d],blockPos:[8,1,${LANE_Z}],nbt:{id:"minecraft:item",` +
      `Pos:[8.5d,1.4d,${LANE_Z}.5d],Motion:[0.0d,0.0d,0.0d],Item:{id:"minecraft:red_shulker_box",Count:1b}}}`,
  ].join(",");
  const snbt = eng.TickSimulation
    .gametestSnbt(schem)
    .replace(/entities:\s*\[\s*\]/, `entities: [${entities}]`);
  if (!/entities:\s*\[\{/.test(snbt)) throw new Error("entity injection failed");

  // Prove the engine actually spawned them before spending a browser on it.
  const sim = eng.TickSimulation.fromSnbt(snbt, eng.TickSettleMode.InWorld, 0, 0, 0, "");
  const at = () => JSON.parse(sim.itemEntitiesJson());
  const t0 = at();
  for (let i = 0; i < 20; i++) sim.step();
  const t20 = at();
  const path = join(WORK, "entities-fixture.snbt");
  writeFileSync(path, snbt);
  return {
    path,
    engine: {
      carts: t0.minecarts.length,
      items: t0.items.length,
      cartX0: t0.minecarts[0]?.pos[0],
      cartX20: t20.minecarts[0]?.pos[0],
      cartY: t20.minecarts[0]?.pos[1],
    },
  };
}

/** The small fixture: the 2x2 flush door with a rail lane along its roof.
 *
 *  `y=6, z=2` is the one plane of this file that is completely empty, so the
 *  lane displaces nothing, and being on top means no block stands between the
 *  camera and the cart. The items land on the roof beside it. */
async function buildSmallFixture() {
  const eng = await import(join(ROOT, "public/engine/index.mjs"));
  const src = `${HOME}/Downloads/2x2-flush-seamless-piston-door.schem`;
  if (!existsSync(src)) throw new Error(`corpus door missing: ${src}`);
  const schem = eng.Schematic.fromData(
    Array.from(new Uint8Array(readFileSync(src))),
  );
  for (let x = 0; x < 6; x++)
    schem.setBlockFromString(x, 6, 2, "minecraft:rail[shape=east_west,waterlogged=false]");
  const entities = [
    `{pos:[0.5d,6.0d,2.5d],blockPos:[0,6,2],nbt:{id:"minecraft:minecart",` +
      `Pos:[0.5d,6.0d,2.5d],Motion:[0.12d,0.0d,0.0d],Rotation:[0.0f,0.0f]}}`,
    `{pos:[3.5d,7.2d,2.5d],blockPos:[3,7,2],nbt:{id:"minecraft:item",` +
      `Pos:[3.5d,7.2d,2.5d],Motion:[0.0d,0.0d,0.0d],Item:{id:"minecraft:diamond",Count:5b}}}`,
    `{pos:[4.5d,7.2d,2.5d],blockPos:[4,7,2],nbt:{id:"minecraft:item",` +
      `Pos:[4.5d,7.2d,2.5d],Motion:[0.0d,0.0d,0.0d],Item:{id:"minecraft:red_shulker_box",Count:1b}}}`,
  ].join(",");
  const snbt = eng.TickSimulation
    .gametestSnbt(schem)
    .replace(/entities:\s*\[\s*\]/, `entities: [${entities}]`);
  const path = join(WORK, "entities-fixture-small.snbt");
  writeFileSync(path, snbt);
  return { path };
}

/* ----------------------------------------------------------- the browser -- */

const browser = await chromium.launch();

async function open(page, file) {
  await page.goto(URL, { waitUntil: "networkidle" });
  await page.locator('input[type="file"]').setInputFiles(file);
  const certified = await Promise.race([
    page.waitForURL(/\/door\//, { timeout: 300_000 }).then(() => true),
    page
      .locator('[data-testid="upload-error"]')
      .waitFor({ timeout: 300_000 })
      .then(() => false),
  ]).catch(() => page.url().includes("/door/"));
  if (!certified) {
    const msg = await page
      .locator('[data-testid="upload-error"] .upload-error-msg')
      .textContent()
      .catch(() => null);
    return { outcome: "refused", message: msg?.replace(/\s+/g, " ").trim() ?? null };
  }
  await page
    .locator('[data-testid="mesh-replay-stage"][data-ready="1"]')
    .waitFor({ timeout: 240_000 })
    .catch(() => {});
  return { outcome: "certified" };
}

function readCert(page) {
  return page.evaluate(() => {
    const key = Object.keys(localStorage).find((k) => k.startsWith("door-cert-wasm:"));
    const rec = JSON.parse(localStorage.getItem(key));
    const c = rec.certificate;
    return {
      verdict: c.verdict,
      cells: c.aperture?.cells ?? null,
      classification: c.classification?.name ?? null,
      open_ticks: c.open_ticks,
      close_ticks: c.close_ticks,
      simTicks: rec.replay.simTicks,
      entities: (rec.replay.entities ?? []).map((e) => ({
        id: e.id,
        kind: e.kind,
        cart: e.cart,
        count: e.count,
        samples: e.track.filter(Boolean).length,
        first: e.track.find(Boolean) ?? null,
        last: [...e.track].reverse().find(Boolean) ?? null,
      })),
    };
  });
}

/** Seek the paused replay to a tick (uncontrolled range, poked with `input`). */
async function seek(page, t) {
  await page.evaluate((v) => {
    const el = document.querySelector('.replay-track input[type="range"]');
    const setter = Object.getOwnPropertyDescriptor(
      window.HTMLInputElement.prototype,
      "value",
    ).set;
    setter.call(el, String(v));
    el.dispatchEvent(new Event("input", { bubbles: true }));
  }, t);
  await page.waitForTimeout(220);
}

const report = {};

/* -- 1. the entity fixture ------------------------------------------------ */

const fixture = await buildFixture();
report.fixture = fixture.engine;

{
  const page = await browser.newPage({ viewport: { width: 1280, height: 1100 } });
  const logs = [];
  page.on("console", (m) => logs.push(`${m.type()}: ${m.text()}`.slice(0, 200)));
  page.on("pageerror", (e) => logs.push(`pageerror: ${String(e).slice(0, 200)}`));

  const res = await open(page, fixture.path);
  report.entityRun = res;
  if (res.outcome === "certified") {
    Object.assign(report.entityRun, await readCert(page));
    await page.getByRole("button", { name: "Pause replay" }).click();
    // Wide shot first, then the cart at three points of its roll so the
    // screenshots show it MOVING and not just present.
    const ticks = [0, 6, 12, 20];
    for (const t of ticks) {
      await seek(page, t);
      await page
        .getByTestId("mesh-replay-stage")
        .screenshot({ path: join(SHOTS, `entities-t${String(t).padStart(3, "0")}.png`) });
    }
    await page.screenshot({ path: join(SHOTS, "entities-page.png"), fullPage: true });
  }
  report.entityRun.logs = logs
    .filter((l) => /error|warn|entity|mesh/i.test(l))
    .slice(0, 12);
  await page.close();
}

/* -- 1b. the entities, close enough to judge ------------------------------ */
//
// The 4x4 shots prove the entities are there and moving, but a 16-wide build
// frames them too small to say whether the MODELS are right. So the second
// fixture is the 2x2 flush door — 6 x 7 x 3, which the replay's own camera
// fits tightly — with the rail lane laid along its ROOF, where nothing can
// occlude it. Captured at 3x device scale at three ticks: a wrong model, a
// missing texture or a half-block offset would all be plain in these.
const small = await buildSmallFixture();
{
  const page = await browser.newPage({
    viewport: { width: 1280, height: 1000 },
    deviceScaleFactor: 3,
  });
  const res = await open(page, small.path);
  report.closeup = res;
  if (res.outcome === "certified") {
    Object.assign(report.closeup, await readCert(page));
    await page.getByRole("button", { name: "Pause replay" }).click();
    const stage = page.getByTestId("mesh-replay-stage");
    const box = await stage.boundingBox();
    await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
    for (let i = 0; i < 5; i++) await page.mouse.wheel(0, -120);
    await page.waitForTimeout(400);
    for (const t of [1, 7, 13]) {
      await seek(page, t);
      await stage.screenshot({
        path: join(SHOTS, `entities-cart-t${String(t).padStart(3, "0")}.png`),
      });
    }
  }
  await page.close();
}

/* -- 2. the certified regression set -------------------------------------- */

const REGRESSION = [
  { slug: "door6x6", file: "6x6 sliding door.litematic", want: { cells: 36, verdict: "CERTIFIED" } },
  { slug: "door4x4", file: "4x4 sliding door.litematic", want: { cells: 16, verdict: "CERTIFIED" } },
  { slug: "vault4x4-filled", file: "fast 4x4 vault door (barrels filled).litematic", want: { cells: 16, verdict: "CERTIFIED" } },
  { slug: "flush2x2", file: "2x2-flush-seamless-piston-door.schem", want: { cells: 4 } },
  { slug: "flush2x2-dupe", file: "2x2-flush-seamless-piston-door (1).schem", want: { cells: 4 } },
  { slug: "funnel4x4", file: "0.45_4x4_funnel.litematic", want: {} },
];

report.regression = {};
for (const d of REGRESSION) {
  const path = `${HOME}/Downloads/${d.file}`;
  if (!existsSync(path)) {
    report.regression[d.slug] = { MISSING: path };
    continue;
  }
  process.stderr.write(`... ${d.slug}\n`);
  const page = await browser.newPage({ viewport: { width: 1280, height: 1100 } });
  const res = await open(page, path);
  const got = res.outcome === "certified" ? { ...res, ...(await readCert(page)) } : res;
  got.OK =
    got.outcome === "certified" &&
    (d.want.cells === undefined || got.cells === d.want.cells) &&
    (d.want.verdict === undefined || got.verdict === d.want.verdict) &&
    // The whole safety claim in one assertion: no entities, no sampling pass.
    (got.entities?.length ?? 0) === 0;
  report.regression[d.slug] = got;
  await page.close();
}

await browser.close();
writeFileSync(join(ROOT, "verify-entities-out.json"), JSON.stringify(report, null, 1));
console.log(JSON.stringify(report, null, 1));
