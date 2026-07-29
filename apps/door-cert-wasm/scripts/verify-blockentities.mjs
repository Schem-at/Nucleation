/** The block-entity audit, checked end to end.
 *
 * Some export tools drop block entities. The blocks all survive, so the build
 * loads clean and every geometric measurement is right — but a comparator with
 * no stored `OutputSignal` reads 0, and the door quietly fails to reset. The
 * validator used to stamp a confident verdict on exactly that file.
 *
 * The twin pair proves it: `0.45_4x4_funnel.litematic` carries 9 block
 * entities, and its `-converted.schem` twin carries none of them. Same door,
 * same blocks. So:
 *
 * 1. The converted twin must read INCONCLUSIVE, and the sheet must carry the
 *    audit naming WHAT is absent and how many — not a bare flag.
 * 2. The litematic twin, which is complete, must be unaffected by any of it.
 * 3. The three doors the standard is calibrated against must not move: same
 *    verdict, same classification, same cell count.
 *
 * Run: npm run build && npx vite preview --port 8455 --strictPort
 *      node scripts/verify-blockentities.mjs
 */
import { chromium } from "playwright";
import { mkdirSync, writeFileSync, existsSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const SHOTS = join(ROOT, "screenshots");
mkdirSync(SHOTS, { recursive: true });
const HOME = process.env.HOME;
const PORT = process.env.PORT ?? "8455";
const URL = `http://localhost:${PORT}/`;

/** The twin pair. Same door, exported twice; one export kept its block
 *  entities and one did not. */
const TWINS = [
  { slug: "funnel-litematic", file: "0.45_4x4_funnel.litematic", want: { missing: 0 } },
  { slug: "funnel-converted", file: "0.45_4x4_funnel-converted.schem", want: { missing: 8 } },
];

/** Must not change: the three doors the standard is calibrated against. */
const REGRESSION = [
  { slug: "door6x6", file: "6x6 sliding door.litematic", want: { cls: "6 × 6 Regular Door", cells: 36, open: 7, close: 7 } },
  { slug: "door4x4", file: "4x4 sliding door.litematic", want: { cls: "4 × 4 Regular Door", cells: 16, open: 4, close: 4 } },
  { slug: "vault-filled", file: "fast 4x4 vault door (barrels filled).litematic", want: { cls: "4 × 4 Vault Door", cells: 16 } },
];

const browser = await chromium.launch();
const report = {};

async function run({ slug, file }) {
  const path = `${HOME}/Downloads/${file}`;
  if (!existsSync(path)) return { MISSING: path };
  const page = await browser.newPage({ viewport: { width: 1280, height: 1200 } });
  const logs = [];
  page.on("console", (m) => logs.push(`${m.type()}: ${m.text()}`.slice(0, 240)));
  page.on("pageerror", (e) => logs.push(`pageerror: ${String(e).slice(0, 200)}`));
  await page.goto(URL, { waitUntil: "networkidle" });
  await page.locator('input[type="file"]').setInputFiles(path);

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
      return {
        outcome: "certified",
        verdict: c.verdict,
        aperture_cells: c.aperture?.cells ?? null,
        classification: c.classification?.name ?? null,
        open_ticks: c.open_ticks,
        close_ticks: c.close_ticks,
        aperture_conflict: !!c.aperture_conflict,
        // The audit as it rode on the record, and as it was rendered.
        audit: c.block_entity_audit ?? null,
        ui: {
          band: q(".sheet-band"),
          audit: q('[data-testid="blockentity-audit"]'),
          aperture: q('[data-testid="aperture"]'),
        },
      };
    });
  }
  out.logs = logs.filter((l) => /error|warn/i.test(l)).slice(0, 6);
  await page.screenshot({ path: join(SHOTS, `${slug}-be.png`), fullPage: out.outcome === "certified" });
  await page.close();
  return out;
}

for (const d of [...TWINS, ...REGRESSION]) {
  process.stderr.write(`... ${d.slug}\n`);
  report[d.slug] = await run(d);
}

// ---- the checks, stated as assertions so the output reads as a verdict ----
const t = (slug) => report[slug] ?? {};
const checks = {};

// 1. The complete twin is untouched: it has no missing block entities and gets
//    a real verdict.
{
  const g = t("funnel-litematic");
  checks.litematic_twin_unaffected =
    g.outcome === "certified" &&
    g.audit?.missing_total === 0 &&
    g.verdict !== "INCONCLUSIVE" &&
    g.ui?.audit === null;
}
// 2. The stripped twin is forced INCONCLUSIVE, the audit rode on the record,
//    and the sheet NAMES the kinds that are absent with their counts.
{
  const g = t("funnel-converted");
  const text = g.ui?.audit ?? "";
  checks.converted_twin_inconclusive = g.outcome === "certified" && g.verdict === "INCONCLUSIVE";
  checks.converted_twin_audit_on_record = (g.audit?.missing_total ?? 0) > 0;
  checks.converted_twin_audit_rendered = text.length > 0;
  // Named, not merely counted: every kind in the audit appears in the copy
  // with its count beside it.
  checks.converted_twin_names_kinds =
    (g.audit?.missing ?? []).length > 0 &&
    (g.audit?.missing ?? []).every((m) => {
      const short = m.name.replace(/^minecraft:/, "").replace(/_/g, " ");
      return text.includes(short) && new RegExp(`${m.count}\\s+${short}`).test(text);
    });
  // The reason must point at the FILE, not at the door.
  checks.converted_twin_blames_the_file = /file|export|saved|carr/i.test(text);
}
// 3. The regression trio: verdict, classification and cell count unchanged.
for (const r of REGRESSION) {
  const g = t(r.slug);
  checks[`regression_${r.slug}`] =
    g.outcome === "certified" &&
    g.verdict === "CERTIFIED" &&
    g.classification === r.want.cls &&
    g.aperture_cells === r.want.cells &&
    (g.audit?.missing_total ?? 0) === 0 &&
    (r.want.open === undefined || g.open_ticks === r.want.open) &&
    (r.want.close === undefined || g.close_ticks === r.want.close);
}

report.CHECKS = checks;
report.ALL_PASS = Object.values(checks).every(Boolean);

await browser.close();
writeFileSync(join(ROOT, "verify-blockentities-out.json"), JSON.stringify(report, null, 1));
console.log(JSON.stringify({ CHECKS: checks, ALL_PASS: report.ALL_PASS }, null, 1));
