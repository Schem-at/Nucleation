/** Doorway timing + reset-time verification against `vite preview` on :8433.
 *
 * Open and close are now measured at the DOORWAY — the first tick every
 * passage cell is air, and the first tick every cell of the closed pattern is
 * solid again — rather than at the tick the machine stops moving. Settle time
 * survives under its own name, and the two must not have swapped places: a
 * doorway time can never exceed the settle time it sits inside.
 *
 * Reset time is Purplers' trial search, measured in both directions.
 *
 * Run: npx vite preview --host --port 8433 --strictPort
 *      node scripts/verify-timing.mjs
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

  const t0 = Date.now();
  await page.goto(URL, { waitUntil: "networkidle" });
  await page.locator('input[type="file"]').setInputFiles(door.file);
  await page.waitForURL(/\/door\//, { timeout: 300_000 });
  const certifyMs = Date.now() - t0;
  await page
    .locator('[data-testid="mesh-replay-stage"][data-ready="1"]')
    .waitFor({ timeout: 180_000 });

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

  const tiles = await page.$$eval(".tile", (els) =>
    els.map((e) => ({
      label: e.querySelector(".tile-label")?.textContent?.trim(),
      value: e.querySelector(".tile-value")?.textContent?.trim(),
      sub: e.querySelector(".tile-sub")?.textContent?.trim(),
    })),
  );
  const sealText = (await page.locator(".seal").textContent()) ?? "";
  const quiet = await page.$$eval(".annot-label", (els) =>
    els.map((e) => e.textContent?.trim()).filter((t) => t?.startsWith("quiet at")),
  );

  for (const theme of ["light", "dark"]) {
    await page.evaluate((t) => {
      document.documentElement.dataset.theme = t;
    }, theme);
    await page.waitForTimeout(300);
    await page.screenshot({
      path: join(SHOTS, `certificate-${door.slug}-${theme}-timing.png`),
      fullPage: true,
    });
  }

  const A = [];
  const ok = (name, cond, got) => A.push({ name, pass: !!cond, got });
  const c = cert ?? {};
  ok(
    `aperture ${door.want[0]} x ${door.want[1]}`,
    c.aperture?.w === door.want[0] && c.aperture?.h === door.want[1],
    c.aperture,
  );
  ok(
    "open <= open settle",
    c.open_ticks === null || (c.open_settle_ticks !== null && c.open_ticks <= c.open_settle_ticks),
    [c.open_ticks, c.open_settle_ticks],
  );
  ok(
    "close <= close settle",
    c.close_ticks === null ||
      (c.close_settle_ticks !== null && c.close_ticks <= c.close_settle_ticks),
    [c.close_ticks, c.close_settle_ticks],
  );
  ok(
    "reset after opening resolved",
    c.reset_open != null && (c.reset_open.ticks !== null || c.reset_open.note !== null),
    c.reset_open,
  );
  ok(
    "reset after closing resolved",
    c.reset_close != null && (c.reset_close.ticks !== null || c.reset_close.note !== null),
    c.reset_close,
  );
  ok("no console errors", consoleErrors.length === 0, consoleErrors);

  report[door.slug] = {
    certifyMs,
    verdict: c.verdict,
    rest_is_closed: c.rest_is_closed,
    needed_priming: c.needed_priming,
    aperture: c.aperture,
    open_ticks: c.open_ticks,
    close_ticks: c.close_ticks,
    open_settle_ticks: c.open_settle_ticks,
    close_settle_ticks: c.close_settle_ticks,
    open_latency: c.open_latency,
    close_latency: c.close_latency,
    timing_note: c.timing_note,
    reset_open: c.reset_open,
    reset_close: c.reset_close,
    cycles_per_minute: c.cycles_per_minute,
    sim_ticks: c.sim_ticks,
    seal: sealText.replace(/\s+/g, " ").trim(),
    quiet_markers: quiet,
    tiles,
    consoleErrors,
    assertions: A,
  };
  await page.close();
}

await browser.close();
writeFileSync(join(ROOT, "verify-timing-out.json"), JSON.stringify(report, null, 2));

const cell = (v) => (v === null || v === undefined ? "—" : String(v));
console.log(
  "\ndoor        aperture  open  close  settle(o/c)  reset-open  reset-close  cert(s)",
);
for (const [slug, r] of Object.entries(report)) {
  const ap = r.aperture ? `${r.aperture.w}x${r.aperture.h}` : "—";
  console.log(
    [
      slug.padEnd(11),
      ap.padEnd(9),
      cell(r.open_ticks).padEnd(5),
      cell(r.close_ticks).padEnd(6),
      `${cell(r.open_settle_ticks)}/${cell(r.close_settle_ticks)}`.padEnd(12),
      cell(r.reset_open?.ticks).padEnd(11),
      cell(r.reset_close?.ticks).padEnd(12),
      (r.certifyMs / 1000).toFixed(1),
    ].join(" "),
  );
}

for (const [slug, r] of Object.entries(report)) {
  console.log(`\n=== ${slug} ===`);
  for (const a of r.assertions)
    console.log(`${a.pass ? "PASS" : "FAIL"}  ${a.name}  ${a.pass ? "" : JSON.stringify(a.got)}`);
  console.log(
    `rest_is_closed=${r.rest_is_closed} needed_priming=${r.needed_priming} timing_note=${r.timing_note}`,
  );
  console.log(`reset_open:  ${JSON.stringify(r.reset_open)}`);
  console.log(`reset_close: ${JSON.stringify(r.reset_close)}`);
  console.log(`quiet markers: ${JSON.stringify(r.quiet_markers)}`);
  console.log(`seal: ${r.seal}`);
  for (const t of r.tiles) console.log(`  tile ${t.label}: ${t.value} — ${t.sub}`);
}
const failed = Object.values(report).flatMap((r) => r.assertions.filter((a) => !a.pass));
console.log(`\n${failed.length === 0 ? "ALL ASSERTIONS PASS" : `${failed.length} FAILURES`}`);
