/** Round-4 (b): mutation-rate schedules. Exponential half-life 10 run —
 * the effective rate decays (sparkline shows it); a mid-run slider move
 * logs "mutation schedule overridden @ gen N" and pins the rate.
 * Screenshots: schedule-sparkline-round4-*, override-event-round4-*.
 * Run: node scripts/verify-round4-schedule.mjs */

import { chromium } from "playwright";
import { join } from "node:path";
import { withPreview, helpers, ok, SHOTS, URL } from "./round4-lib.mjs";

const parseNow = (t) => parseFloat(t.replace(/.*now\s+/, ""));

await withPreview(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width: 1480, height: 1050 } });
  const h = helpers(page);
  await page.goto(URL, { waitUntil: "networkidle" });

  await page.locator("#cfg-pop").fill("64");
  await h.openGroup("schedules");
  await page.getByTestId("cfg-mut-schedule").selectOption("exponential");
  await page.getByTestId("cfg-mut-halflife").fill("10");
  await page.getByTestId("start-run").click();
  await page
    .getByTestId("stat-status")
    .filter({ hasText: "running" })
    .waitFor({ timeout: 90_000 });

  await h.waitGen(20, 240_000);
  const now1 = parseNow(await page.getByTestId("rate-now").innerText());
  // base 0.05, half-life 10 -> ~0.0125 at gen 20; anything < 60% of base
  // proves a real decay.
  ok("schedule-decays", now1 < 0.05 * 0.6, `rate now ${now1} (base 0.050, hl 10, gen ≥ 20)`);
  ok(
    "sparkline-present",
    (await page.getByTestId("rate-sparkline").count()) === 1,
    "rate sparkline rendered",
  );
  const spark = page.getByTestId("group-schedules");
  await spark.screenshot({ path: join(SHOTS, "schedule-sparkline-round4-light.png") });
  await h.toggleTheme();
  await page.waitForTimeout(250);
  await spark.screenshot({ path: join(SHOTS, "schedule-sparkline-round4-dark.png") });
  await h.toggleTheme();
  await page.waitForTimeout(250);

  // Manual override: slider 0.05 -> 0.10 via keyboard (10 × step 0.005).
  const slider = page.getByTestId("cfg-mutrate");
  await slider.focus();
  for (let i = 0; i < 10; i++) await page.keyboard.press("ArrowRight");
  await page.waitForTimeout(4000); // debounce + ≥1 generation boundary
  const ev = await h.eventsText();
  ok(
    "override-event",
    ev.includes("mutation schedule overridden"),
    `events feed ${ev.includes("mutation schedule overridden") ? "contains" : "MISSING"} "mutation schedule overridden @ gen N"`,
  );
  const pinned = parseFloat(await slider.inputValue());
  const nowA = parseNow(await page.getByTestId("rate-now").innerText());
  await page.waitForTimeout(3500); // a couple more generations
  const nowB = parseNow(await page.getByTestId("rate-now").innerText());
  ok(
    "rate-pinned",
    Math.abs(nowA - pinned) < 1e-9 && Math.abs(nowB - pinned) < 1e-9,
    `rate pinned at ${pinned}: samples ${nowA}, ${nowB}`,
  );

  const feed = page.getByTestId("events-feed");
  await feed.screenshot({ path: join(SHOTS, "override-event-round4-light.png") });
  await h.toggleTheme();
  await page.waitForTimeout(250);
  await feed.screenshot({ path: join(SHOTS, "override-event-round4-dark.png") });

  await page.getByTestId("stop-run").click();
  await page
    .getByTestId("stat-status")
    .filter({ hasText: "done" })
    .waitFor({ timeout: 120_000 });
  await browser.close();
});
