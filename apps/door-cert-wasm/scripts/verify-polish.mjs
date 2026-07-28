/** Polish-pass verification against `vite preview` on :8433.
 *
 * For each door: upload, wait for the WebGL replay to finish meshing, then
 * assert the certificate's structural promises — exactly two lever flips in
 * the replay, the scrubber starting at the open click, an aperture read off
 * the sheet, real (non-zero-size) textures in the materials grid, and no
 * console errors. Captures full-page light + dark screenshots.
 *
 * Run: node scripts/verify-polish.mjs
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

  // Pause so screenshots and DOM reads are stable.
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
    return recs.length ? recs[recs.length - 1] : null;
  });

  const marks = await page.locator(".replay-mark").evaluateAll((els) =>
    els.map((e) => ({ title: e.getAttribute("title"), left: e.style.left })),
  );

  const textures = await page.locator(".stack-tex").evaluateAll((els) =>
    els.map((e) => ({
      img: e.tagName === "IMG",
      w: e.naturalWidth ?? 0,
      h: e.naturalHeight ?? 0,
      label: e.closest(".stack")?.querySelector(".stack-name")?.textContent ?? "",
    })),
  );

  const apertureText = (await page.getByTestId("aperture").textContent()) ?? "";
  const censusText = (await page.getByTestId("census").textContent().catch(() => "")) ?? "";
  const speedButtons = await page.locator(".replay-speed").allTextContents();
  const hasIsoToggle = await page.locator('[data-testid="replay-mode-toggle"]').count();

  // Screenshots, both themes.
  for (const theme of ["light", "dark"]) {
    await page.evaluate((t) => {
      document.documentElement.dataset.theme = t;
    }, theme);
    await page.waitForTimeout(300);
    await page.screenshot({
      path: join(SHOTS, `certificate-${door.slug}-${theme}-polish.png`),
      fullPage: true,
    });
  }

  const c = cert?.certificate ?? {};
  const r = cert?.replay ?? {};
  report[door.slug] = {
    verdict: c.verdict,
    open_ticks: c.open_ticks,
    close_ticks: c.close_ticks,
    open_latency: c.open_latency,
    close_latency: c.close_latency,
    moved_cells: c.moved_cells,
    paste_safe: c.paste_safe,
    paste_moved_cells: c.paste_moved_cells,
    needed_priming: c.needed_priming,
    aperture: c.aperture,
    aperture_text: apertureText.replace(/\s+/g, " ").trim(),
    peak_changes: c.peak_changes,
    peak_tick: c.peak_tick,
    cycles_per_minute: c.cycles_per_minute,
    volume: c.volume,
    dims: c.dims,
    sim_ticks: c.sim_ticks,
    census: c.census,
    census_text: censusText.replace(/\s+/g, " ").trim(),
    materials_kinds: (c.materials ?? []).length,
    replay_flips: r.flips,
    replay_first_change_tick: (r.changes ?? []).reduce(
      (m, ch) => Math.min(m, ch.tick),
      Infinity,
    ),
    marks,
    speedButtons,
    hasIsoToggle,
    textures_total: textures.length,
    textures_missing: textures.filter((t) => !t.img).map((t) => t.label),
    textures_zero_size: textures.filter((t) => t.img && (t.w === 0 || t.h === 0)),
    consoleErrors,
  };

  // -- assertions ---------------------------------------------------------
  const A = [];
  const ok = (name, cond, got) => A.push({ name, pass: !!cond, got });
  ok("exactly 2 lever flips", (r.flips ?? []).length === 2, (r.flips ?? []).length);
  ok("all flips measured", (r.flips ?? []).every((f) => f.measured), r.flips);
  ok("exactly 2 scrubber marks", marks.length === 2, marks.length);
  ok("scrubber starts at open click", (r.flips ?? [])[0]?.tick === 0, (r.flips ?? [])[0]);
  ok("first mark at 0%", marks[0]?.left === "0%", marks[0]?.left);
  ok("no iso toggle", hasIsoToggle === 0, hasIsoToggle);
  ok("speeds 0.5/1/2/4 visible", speedButtons.join(",") === "0.5×,1×,2×,4×", speedButtons);
  ok("aperture reported", !!c.aperture, c.aperture);
  ok("materials grid rendered", textures.length > 0, textures.length);
  ok(
    "every stack has a real texture",
    textures.length > 0 && textures.every((t) => t.img && t.w > 0 && t.h > 0),
    textures.filter((t) => !(t.img && t.w > 0)).map((t) => t.label),
  );
  ok("no console errors", consoleErrors.length === 0, consoleErrors);
  if (door.slug === "door6x6") {
    ok("open 15", c.open_ticks === 15, c.open_ticks);
    ok("close 13", c.close_ticks === 13, c.close_ticks);
    ok("stroke 760", c.moved_cells === 760, c.moved_cells);
    ok("CERTIFIED", c.verdict === "CERTIFIED", c.verdict);
    ok("paste_safe false", c.paste_safe === false, c.paste_safe);
    ok("aperture 6x6", c.aperture?.w === 6 && c.aperture?.h === 6, c.aperture);
  }
  report[door.slug].assertions = A;
  await page.close();
}

await browser.close();
writeFileSync(join(ROOT, "verify-polish-out.json"), JSON.stringify(report, null, 2));

for (const [slug, r] of Object.entries(report)) {
  console.log(`\n=== ${slug} ===`);
  for (const a of r.assertions)
    console.log(`${a.pass ? "PASS" : "FAIL"}  ${a.name}  ${a.pass ? "" : JSON.stringify(a.got)}`);
  console.log(
    `open=${r.open_ticks}(lat ${r.open_latency}) close=${r.close_ticks}(lat ${r.close_latency}) ` +
      `stroke=${r.moved_cells} peak=${r.peak_changes}@t${r.peak_tick} cpm=${r.cycles_per_minute?.toFixed(1)} ` +
      `aperture=${JSON.stringify(r.aperture)} vol=${r.volume} dims=${JSON.stringify(r.dims)} ` +
      `cycle=${r.sim_ticks}t priming=${r.needed_priming} pasteSafe=${r.paste_safe}(${r.paste_moved_cells}) ` +
      `kinds=${r.materials_kinds} tex=${r.textures_total}`,
  );
  console.log(`aperture line: ${r.aperture_text}`);
  console.log(`census: ${r.census_text}`);
}
const failed = Object.values(report).flatMap((r) => r.assertions.filter((a) => !a.pass));
console.log(`\n${failed.length === 0 ? "ALL ASSERTIONS PASS" : `${failed.length} FAILURES`}`);
