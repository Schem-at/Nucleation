/** The four fixes from the v2 batch report, checked end to end.
 *
 * 1. Engine failures name the block. `TickSimulation.lastErrorDetail()` is read
 *    in the worker's catch and the offending block is lifted into the sentence.
 * 2. The passage extractor solves every plane axis, not only the densest one,
 *    and a component has to be big enough to walk through. A 2 x 2 seamless
 *    door with its own lever finds its doorway; note-block BUDs do not become
 *    inputs, because an input has to swing the doorway and hold it.
 * 3. An INCONCLUSIVE sheet carries no timing in its hero and no timing tile at
 *    certified size.
 * 4. A blockstate the mesher cannot encode falls back to the bare block rather
 *    than leaving a hole; the size guard quotes one number in one unit.
 *
 * Run: npm run build && npx vite preview --host --port 8433 --strictPort
 *      node scripts/verify-fixes.mjs
 */
import { chromium } from "playwright";
import { mkdirSync, writeFileSync, existsSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const SHOTS = join(ROOT, "screenshots");
mkdirSync(SHOTS, { recursive: true });
const HOME = process.env.HOME;
const URL = "http://localhost:8433/";

/** Must not change: the three doors the standard is calibrated against. */
const REGRESSION = [
  { slug: "door6x6", file: "6x6 sliding door.litematic", want: { cls: "6 × 6 Regular Door", cells: 36 } },
  { slug: "door4x4", file: "4x4 sliding door.litematic", want: { cls: "4 × 4 Regular Door", cells: 16 } },
  { slug: "vault-filled", file: "fast 4x4 vault door (barrels filled).litematic", want: { cls: "4 × 4 Vault Door", cells: 16 } },
];
/** The five that reported "no passage anyone could walk through". */
const NO_PASSAGE = [
  { slug: "seamless2x2", file: "2x2-flush-seamless-piston-door.schem" },
  { slug: "seamless2x2-dupe", file: "2x2-flush-seamless-piston-door (1).schem" },
  { slug: "matheus330b", file: "330b_unseamless_5x5_by_Matheus.litematic" },
  { slug: "funni3x3", file: "funni3x3deano.litematic" },
  { slug: "skittles", file: "skittles-270b-3x3hipster.schem" },
];
/** One of each remaining shape of failure. */
const OTHER = [
  { slug: "inconclusive-vault", file: "fast 4x4 vault door.litematic", expect: "certificate" },
  { slug: "chain-replay", file: "780b_0.6s_unseamless_5x5.litematic", expect: "certificate" },
  // Every door in the v2 batch now loads, so the engine-error path is proved on
  // a file that still does not: this one is refused for `minecraft:quartz_slab`
  // and `minecraft:waxed_copper_block`, and the sentence must say so.
  { slug: "unsupported-block", file: "0b576cd5-156c-485d-a566-24f6a3f1294e.schem", expect: "error" },
  { slug: "legacy-ids", file: "3x7_bore_by_aqkrm.litematic", expect: "either" },
  { slug: "size-guard", file: "IRIS_B.schem", expect: "error" },
];

const browser = await chromium.launch();
const report = {};

async function run({ slug, file, expect = "either" }) {
  const path = `${HOME}/Downloads/${file}`;
  if (!existsSync(path)) return { MISSING: path };
  const page = await browser.newPage({ viewport: { width: 1280, height: 1200 } });
  const logs = [];
  page.on("console", (m) => logs.push(`${m.type()}: ${m.text()}`.slice(0, 240)));
  page.on("pageerror", (e) => logs.push(`pageerror: ${String(e).slice(0, 200)}`));
  await page.goto(URL, { waitUntil: "networkidle" });
  await page.locator('input[type="file"]').setInputFiles(path);

  // Race the two outcomes: waiting out the certificate timeout on every
  // refusal turns a nine-file run into an hour of nothing happening.
  const certified = await Promise.race([
    page.waitForURL(/\/door\//, { timeout: 300_000 }).then(() => true),
    page
      .locator('[data-testid="upload-error"]')
      .waitFor({ timeout: 300_000 })
      .then(() => false),
  ]).catch(() => page.url().includes("/door/"));

  let out;
  if (!certified) {
    const box = page.locator('[data-testid="upload-error"]');
    out = {
      outcome: "refused",
      message: await box.locator(".upload-error-msg").textContent().catch(() => null),
      code: await box.locator('[data-testid="upload-error-code"]').textContent().catch(() => null),
    };
    if (out.message) out.message = out.message.replace(/\s+/g, " ").trim();
    if (out.code) out.code = out.code.replace(/\s+/g, " ").trim();
  } else {
    await page
      .locator('[data-testid="mesh-replay-stage"][data-ready="1"]')
      .waitFor({ timeout: 240_000 })
      .catch(() => {});
    out = await page.evaluate(() => {
      const key = Object.keys(localStorage).find((k) => k.startsWith("door-cert-wasm:"));
      const c = JSON.parse(localStorage.getItem(key)).certificate;
      const q = (s) => document.querySelector(s)?.textContent?.replace(/\s+/g, " ").trim() ?? null;
      const tiles = [...document.querySelectorAll(".tile")].map((t) => ({
        label: t.querySelector(".tile-label")?.textContent?.replace(/\s+/g, " ").trim(),
        value: t.querySelector(".tile-value")?.textContent?.trim(),
        px: parseFloat(getComputedStyle(t.querySelector(".tile-value")).fontSize),
        disputed: t.classList.contains("tile-disputed"),
      }));
      return {
        outcome: "certified",
        verdict: c.verdict,
        aperture: c.aperture,
        classification: c.classification?.name ?? null,
        open_ticks: c.open_ticks,
        close_ticks: c.close_ticks,
        input: c.input && { kind: c.input.kind, pos: c.input.pos, moved: c.input.moved },
        input_note: c.input_note,
        aperture_conflict: !!c.aperture_conflict,
        ui: {
          heroDims: q(".hero-dims"),
          aperture: q('[data-testid="aperture"]'),
          inputNote: q('[data-testid="input-note"]'),
          caveat: q('[data-testid="measurements-caveat"]'),
          unmeshed: q('[data-testid="unmeshed"]'),
          maxTilePx: Math.max(...tiles.map((t) => t.px)),
          disputedTiles: tiles.filter((t) => t.disputed).length,
          tiles: tiles.length,
        },
      };
    });
  }
  out.logs = logs.filter((l) => /error|warn|mesh/i.test(l)).slice(0, 8);
  out.expectationMet = expect === "either" || (expect === "certificate") === (out.outcome === "certified");
  await page.screenshot({ path: join(SHOTS, `${slug}-fixes.png`), fullPage: out.outcome === "certified" });
  await page.close();
  return out;
}

for (const d of [...REGRESSION, ...NO_PASSAGE, ...OTHER]) {
  process.stderr.write(`... ${d.slug}\n`);
  report[d.slug] = await run(d);
  const w = REGRESSION.find((r) => r.slug === d.slug)?.want;
  if (w) {
    const got = report[d.slug];
    report[d.slug].REGRESSION_OK =
      got.verdict === "CERTIFIED" &&
      got.classification === w.cls &&
      got.aperture?.cells === w.cells;
  }
}

await browser.close();
writeFileSync(join(ROOT, "verify-fixes-out.json"), JSON.stringify(report, null, 1));
console.log(JSON.stringify(report, null, 1));
