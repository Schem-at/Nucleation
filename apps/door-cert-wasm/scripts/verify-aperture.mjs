/** Aperture / pattern extraction verification against `vite preview` on :8433.
 *
 * The doorway is measured from the passage that opens, not from the set of
 * cells that changed (see src/lib/aperture.ts). This checks that on real
 * files: the 4x4 vault reads 4 x 4 rather than the 6 x 5 halo the changed-set
 * produced, and the two plain doors are unmoved.
 *
 * Run: npx vite preview --host --port 8433 --strictPort
 *      node scripts/verify-aperture.mjs
 */

import { chromium } from "playwright";
import { mkdirSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const SHOTS = join(ROOT, "screenshots");
mkdirSync(SHOTS, { recursive: true });

const URL = "http://localhost:8433/";
const DOORS = [
  {
    slug: "vault4x4",
    file: "/Users/harrison/Downloads/fast 4x4 vault door (barrels filled).litematic",
    want: [4, 4],
  },
  { slug: "door6x6", file: "/tmp/door6x6.litematic", want: [6, 6] },
  {
    slug: "door4x4",
    file: join(ROOT, "../../crates/mc-tick/tests/corpus/structures/door_4x4_sliding.snbt"),
    want: [4, 4],
  },
];

const report = {};
const browser = await chromium.launch();

for (const door of DOORS) {
  const consoleErrors = [];
  const page = await browser.newPage({ viewport: { width: 1280, height: 1000 } });
  page.on("console", (m) => {
    if (m.type() === "error") consoleErrors.push(m.text());
  });
  page.on("pageerror", (e) => consoleErrors.push(`pageerror: ${e}`));

  await page.goto(URL, { waitUntil: "networkidle" });
  await page.locator('input[type="file"]').setInputFiles(door.file);
  await page.waitForURL(/\/door\//, { timeout: 180_000 });
  await page
    .locator('[data-testid="mesh-replay-stage"][data-ready="1"]')
    .waitFor({ timeout: 180_000 });
  await page.locator(".replay-btn").first().click();
  await page.waitForTimeout(400);

  const cert = await page.evaluate(() => {
    const recs = Object.keys(localStorage)
      .filter((k) => k.includes("door"))
      .map((k) => {
        try {
          return JSON.parse(localStorage.getItem(k));
        } catch {
          return null;
        }
      })
      .filter((r) => r && r.certificate);
    return recs.length ? recs[recs.length - 1].certificate : null;
  });

  const apertureText = (await page.getByTestId("aperture").textContent()) ?? "";

  for (const theme of ["light", "dark"]) {
    await page.evaluate((t) => {
      document.documentElement.dataset.theme = t;
    }, theme);
    await page.waitForTimeout(300);
    await page.screenshot({
      path: join(SHOTS, `certificate-${door.slug}-${theme}-aperture.png`),
      fullPage: true,
    });
  }

  const cl = cert?.classification ?? null;
  const A = [];
  const ok = (name, cond, got) => A.push({ name, pass: !!cond, got });
  ok(
    `aperture ${door.want[0]} x ${door.want[1]}`,
    cert?.aperture?.w === door.want[0] && cert?.aperture?.h === door.want[1],
    cert?.aperture,
  );
  ok("no console errors", consoleErrors.length === 0, consoleErrors);

  report[door.slug] = {
    verdict: cert?.verdict,
    aperture: cert?.aperture,
    aperture_text: apertureText.replace(/\s+/g, " ").trim(),
    classification: cl && {
      name: cl.name,
      pattern: cl.pattern,
      patternRef: cl.patternRef,
      transform: cl.transform,
      qualifiers: cl.qualifiers,
      frameNote: cl.frameNote,
      layers: cl.layers,
      extruded: cl.extruded,
      unclassified: cl.unclassified,
      composition: cl.composition,
      matrix: cl.matrix,
      depth: cl.depth,
    },
    consoleErrors,
    assertions: A,
  };
  await page.close();
}

await browser.close();
writeFileSync(join(ROOT, "verify-aperture-out.json"), JSON.stringify(report, null, 2));

for (const [slug, r] of Object.entries(report)) {
  console.log(`\n=== ${slug} ===`);
  for (const a of r.assertions)
    console.log(`${a.pass ? "PASS" : "FAIL"}  ${a.name}  ${a.pass ? "" : JSON.stringify(a.got)}`);
  console.log(`verdict=${r.verdict} aperture=${JSON.stringify(r.aperture)}`);
  console.log(`aperture line: ${r.aperture_text}`);
  const c = r.classification;
  if (!c) {
    console.log("no classification");
    continue;
  }
  console.log(
    `name="${c.name}" pattern=${c.pattern} ${c.patternRef ?? ""} transform=${c.transform} ` +
      `layers=${c.layers} extruded=${c.extruded} qualifiers=${JSON.stringify(c.qualifiers)}`,
  );
  console.log(`frameNote: ${c.frameNote}`);
  console.log(`composition: ${JSON.stringify(c.composition.map((t) => t.label))}`);
  console.log("matrix:");
  for (const row of c.matrix) console.log("   " + row.map((v) => (v ? "■" : "□")).join(" "));
  console.log("first layer per cell (. = air):");
  for (const row of c.depth) console.log("   " + row.map((v) => (v < 0 ? "." : v)).join(" "));
}
const failed = Object.values(report).flatMap((r) => r.assertions.filter((a) => !a.pass));
console.log(`\n${failed.length === 0 ? "ALL ASSERTIONS PASS" : `${failed.length} FAILURES`}`);
