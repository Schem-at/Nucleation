/** The four defects the batch run exposed, checked end to end.
 *
 * 1. A door whose saved state and settled cycle open DIFFERENT doorways must
 *    not print a confident classification — `fast 4x4 vault door` (empty
 *    barrels) used to certify a 4 × 1 opening in 1 tick.
 * 2. The input is found by actuating candidates, not by looking for a lever.
 * 3. A non-rectangular opening never lets `w × h` imply a cell count.
 * 4. Engine errors reach the screen as sentences, with the raw code as small
 *    print, and an oversized file is refused before it can OOM the engine.
 *
 * Run: npx vite preview --host --port 8433 --strictPort
 *      node scripts/verify-batchfix.mjs
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

const DOORS = [
  { slug: "vault-empty", file: `${HOME}/Downloads/fast 4x4 vault door.litematic` },
  { slug: "vault-filled", file: `${HOME}/Downloads/fast 4x4 vault door (barrels filled).litematic` },
  { slug: "door6x6", file: `${HOME}/Downloads/6x6 sliding door.litematic` },
  { slug: "door4x4", file: `${HOME}/Downloads/4x4 sliding door.litematic` },
  { slug: "tgm4x4", file: `${HOME}/Downloads/fast tgm 4x4.litematic` },
];
const REFUSALS = [
  { slug: "no-control", file: `${HOME}/Downloads/3x3 flush synced.litematic` },
  { slug: "nothing-moved", file: `${HOME}/Downloads/696c6633-4e8d-4cbb-ac53-80df5847899e.schem` },
  { slug: "unsupported-block", file: `${HOME}/Downloads/3x3_Piston_Door_d24284.litematic` },
  { slug: "size-guard", file: `${HOME}/Downloads/IRIS_B.schem` },
];

const report = { certified: {}, refused: {} };
const browser = await chromium.launch();

for (const door of DOORS) {
  if (!existsSync(door.file)) { report.certified[door.slug] = { MISSING: door.file }; continue; }
  const page = await browser.newPage({ viewport: { width: 1280, height: 1100 } });
  const errs = [];
  page.on("pageerror", (e) => errs.push(String(e)));
  await page.goto(URL, { waitUntil: "networkidle" });
  await page.locator('input[type="file"]').setInputFiles(door.file);
  try {
    await page.waitForURL(/\/door\//, { timeout: 300_000 });
  } catch {
    const msg = await page.locator('[data-testid="upload-error"]').textContent().catch(() => null);
    report.certified[door.slug] = { FAILED_TO_CERTIFY: msg, errs };
    await page.screenshot({ path: join(SHOTS, `${door.slug}-batchfix.png`), fullPage: false });
    await page.close();
    continue;
  }
  await page.locator('[data-testid="mesh-replay-stage"][data-ready="1"]').waitFor({ timeout: 180_000 });

  const out = await page.evaluate(() => {
    const key = Object.keys(localStorage).find((k) => k.startsWith("door-cert-wasm:"));
    const c = JSON.parse(localStorage.getItem(key)).certificate;
    const q = (s) => document.querySelector(s)?.textContent?.replace(/\s+/g, " ").trim() ?? null;
    return {
      verdict: c.verdict,
      aperture: c.aperture,
      classification: c.classification?.name ?? null,
      open_ticks: c.open_ticks,
      close_ticks: c.close_ticks,
      input: c.input && { kind: c.input.kind, pos: c.input.pos, moved: c.input.moved },
      input_alternatives: (c.input_alternatives ?? []).length,
      input_note: c.input_note,
      needed_priming: c.needed_priming,
      saved_state_drift: c.saved_state_drift,
      aperture_conflict: c.aperture_conflict,
      ui: {
        band: q(".sheet-band"),
        aperture: q('[data-testid="aperture"]'),
        conflict: q('[data-testid="conflict"]'),
        seal: document.querySelector(".seal")?.getAttribute("aria-label") ?? null,
        classifyBlock: q('[data-testid="classify-name"]'),
        inputNote: q('[data-testid="input-note"]'),
      },
      errs: [],
    };
  });
  out.errs = errs;
  report.certified[door.slug] = out;
  await page.screenshot({ path: join(SHOTS, `${door.slug}-batchfix.png`), fullPage: true });
  await page.close();
}

for (const r of REFUSALS) {
  if (!existsSync(r.file)) { report.refused[r.slug] = { MISSING: r.file }; continue; }
  const page = await browser.newPage({ viewport: { width: 1280, height: 900 } });
  await page.goto(URL, { waitUntil: "networkidle" });
  await page.locator('input[type="file"]').setInputFiles(r.file);
  const box = page.locator('[data-testid="upload-error"]');
  let res;
  try {
    await box.waitFor({ timeout: 300_000 });
    res = {
      message: (await box.locator(".upload-error-msg").textContent()).replace(/\s+/g, " ").trim(),
      code: await box.locator('[data-testid="upload-error-code"]').textContent().catch(() => null),
      certified: page.url().includes("/door/"),
    };
  } catch (e) {
    res = { NO_ERROR_SHOWN: page.url(), why: String(e).slice(0, 120) };
  }
  report.refused[r.slug] = res;
  await page.screenshot({ path: join(SHOTS, `${r.slug}-batchfix.png`) });
  await page.close();
}

await browser.close();
writeFileSync(join(ROOT, "verify-batchfix-out.json"), JSON.stringify(report, null, 1));
console.log(JSON.stringify(report, null, 1));
