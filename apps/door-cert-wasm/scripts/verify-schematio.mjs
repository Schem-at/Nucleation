/** Verification for the schemat.io re-skin: pattern classification, the
 *  stacked activity trace, and the removal of the paste caveat.
 *
 *  Run against `npx vite preview --host --port 8433 --strictPort`:
 *    node scripts/verify-schematio.mjs
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
  { slug: "door6x6", file: "/tmp/door6x6.litematic" },
  {
    slug: "door4x4",
    file: join(ROOT, "../../crates/mc-tick/tests/corpus/structures/door_4x4_sliding.snbt"),
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
      .map((k) => {
        try {
          return JSON.parse(localStorage.getItem(k));
        } catch {
          return null;
        }
      })
      .filter((r) => r && r.certificate);
    return recs[recs.length - 1].certificate;
  });

  const dom = await page.evaluate(() => {
    const block = document.querySelector('[data-testid="classification"]');
    const matrixCells = block ? block.querySelectorAll(".matrix-fig svg rect").length : 0;
    const chart = document.querySelector(".chart-block .chart-svg-wrap svg");
    const seriesFills = chart
      ? [...chart.querySelectorAll("rect[fill], path[fill]")].map((n) => n.getAttribute("fill"))
      : [];
    return {
      hasClassification: !!block,
      classifyName: block?.querySelector('[data-testid="classify-name"]')?.textContent?.trim(),
      chips: [...(block?.querySelectorAll(".classify-chips .badge") ?? [])].map((n) =>
        n.textContent.trim(),
      ),
      note: block?.querySelector(".classify-note")?.textContent?.trim() ?? null,
      matrixCells,
      hasPasteCaveat: !!document.querySelector(".hero-caveat"),
      pasteCaveatText: document.body.innerText.includes("Needs priming after pasting"),
      legend: [...document.querySelectorAll(".legend .item")].map((n) => n.textContent.trim()),
      pistonBars: seriesFills.filter((f) => f === "var(--series-piston)").length,
      redstoneBars: seriesFills.filter((f) => f === "var(--series-redstone)").length,
      settleLabel:
        [...document.querySelectorAll(".annot-label")]
          .map((n) => n.textContent)
          .find((t) => t.startsWith("quiet")) ?? null,
      callouts: [...document.querySelectorAll(".callout-label")].map((n) => n.textContent),
      fontFamily: getComputedStyle(document.body).fontFamily,
      title: document.title,
    };
  });

  // Hover the busiest tick so the tooltip breakdown is exercised.
  const wrap = page.locator(".chart-block .chart-svg-wrap").first();
  await wrap.scrollIntoViewIfNeeded();
  await page.waitForTimeout(200);
  const box = await wrap.boundingBox();
  await page.mouse.move(box.x + box.width * 0.12, box.y + box.height * 0.6);
  await page.waitForTimeout(150);
  const tooltip = await page.locator(".tooltip").first().innerText().catch(() => null);
  await page.mouse.move(box.x - 50, box.y - 50);

  const assertions = {
    classificationRenders: dom.hasClassification,
    matrixGridRenders: dom.matrixCells > 0,
    noPasteCaveat: !dom.hasPasteCaveat && !dom.pasteCaveatText,
    pasteFieldsStillInJson:
      typeof cert.paste_safe === "boolean" && typeof cert.paste_moved_cells === "number",
    stackedChartHasBothSeries: dom.pistonBars > 0 && dom.redstoneBars > 0,
    itemsSeriesPresent: dom.legend.some((l) => l.startsWith("Item events")),
    tooltipBreakdown: !!tooltip && /Total/.test(tooltip),
    noConsoleErrors: consoleErrors.length === 0,
  };

  report[door.slug] = {
    assertions,
    consoleErrors,
    classification: cert.classification,
    aperture: cert.aperture,
    dom: { ...dom, tooltip },
  };

  for (const theme of ["light", "dark"]) {
    await page.evaluate((t) => {
      document.documentElement.dataset.theme = t;
    }, theme);
    await page.waitForTimeout(300);
    await page.screenshot({
      path: join(SHOTS, `certificate-${door.slug}-${theme}-schematio.png`),
      fullPage: true,
    });
  }

  await page.close();
}

await browser.close();
writeFileSync(join(ROOT, "verify-schematio-out.json"), JSON.stringify(report, null, 2));

for (const [slug, r] of Object.entries(report)) {
  console.log(`\n== ${slug} ==`);
  for (const [k, v] of Object.entries(r.assertions)) console.log(`  ${v ? "PASS" : "FAIL"}  ${k}`);
  const c = r.classification;
  console.log(`  aperture: ${JSON.stringify(r.aperture)}`);
  if (!c) console.log("  classification: null");
  else {
    console.log(`  name: ${c.name}`);
    console.log(
      `  pattern=${c.pattern ?? "UNCLASSIFIED"} ${c.patternRef ?? ""} · orientation=${c.orientation} · qualifiers=${JSON.stringify(c.qualifiers)} · layers=${c.layers}`,
    );
    console.log(`  transform: ${c.transform ?? "none"}`);
    console.log(`  composition: ${JSON.stringify(c.composition)}`);
    console.log("  matrix:");
    for (const row of c.matrix) console.log("    " + row.join(" "));
    console.log("  depth:");
    for (const row of c.depth) console.log("    " + row.map((v) => (v < 0 ? "." : v)).join(" "));
  }
  console.log(`  chart legend: ${JSON.stringify(r.dom.legend)}`);
  console.log(`  settle: ${r.dom.settleLabel} · callouts: ${JSON.stringify(r.dom.callouts)}`);
  if (r.consoleErrors.length) console.log(`  console: ${JSON.stringify(r.consoleErrors)}`);
}
