/** Round-4 (d): population inspector. Under tight constraints (min blocks
 * 6, piston cap 1) a pop-64 run shows exactly 64 cells (virtualized:
 * data-total + last cell reachable by scroll) with ≥ 1 invalid badge;
 * clicking a cell stages it in the machine viewer.
 * Screenshots: population-round4-{light,dark}.
 * Run: node scripts/verify-round4-pop.mjs */

import { chromium } from "playwright";
import { join } from "node:path";
import { withPreview, helpers, ok, SHOTS, URL } from "./round4-lib.mjs";

await withPreview(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width: 1480, height: 1050 } });
  const h = helpers(page);
  await page.goto(URL, { waitUntil: "networkidle" });

  await page.locator("#cfg-pop").fill("64");
  // Tight constraints so some genomes are culled with named violations.
  await h.openGroup("constraints");
  const minb = page.getByTestId("cfg-minb");
  await minb.focus();
  for (let i = 0; i < 5; i++) await page.keyboard.press("ArrowRight"); // 1 -> 6
  await page.locator("#cfg-pistons").fill("1");
  await page.getByTestId("start-run").click();
  await page
    .getByTestId("stat-status")
    .filter({ hasText: "running" })
    .waitFor({ timeout: 90_000 });
  await h.waitGen(3, 120_000);

  await page.getByTestId("tab-population").click();
  await page.getByTestId("population-inspector").waitFor({ timeout: 10_000 });
  const total = await page
    .getByTestId("pop-scroll")
    .getAttribute("data-total");
  ok("pop-total-64", total === "64", `inspector data-total = ${total} (population 64)`);
  const countText = await page.getByTestId("pop-count").innerText();
  ok(
    "pop-count-readout",
    countText.includes("64 machines"),
    `header reads "${countText}"`,
  );
  const invalid = await page.getByTestId("pop-invalid").count();
  ok("pop-invalid-badges", invalid >= 1, `${invalid} invalid badge(s) visible in the viewport`);
  const badgeText = invalid > 0 ? await page.getByTestId("pop-invalid").first().innerText() : "";
  ok(
    "badge-names-constraint",
    /under|piston|blocks|banned|missing|stragglers|stalled|period|flier/.test(badgeText),
    `badge names the culling constraint: "${badgeText}"`,
  );

  // Virtualization: the last cell (#64) is reachable by scroll and the DOM
  // holds fewer cells than the population when scrolled to the top.
  const domCellsTop = await page.getByTestId("pop-cell").count();
  await page
    .getByTestId("pop-scroll")
    .evaluate((el) => (el.scrollTop = el.scrollHeight));
  await page.waitForTimeout(300);
  const last = await page.locator('button[title^="#64 "]').count();
  ok("pop-virtual-last-cell", last === 1, `cell #64 present after scrolling (top DOM held ${domCellsTop} cells)`);
  await page.getByTestId("pop-scroll").evaluate((el) => (el.scrollTop = 0));
  await page.waitForTimeout(300);

  await page
    .locator("section.panel", { hasText: "Population" })
    .first()
    .screenshot({ path: join(SHOTS, "population-round4-light.png") });
  await h.toggleTheme();
  await page.waitForTimeout(250);
  await page
    .locator("section.panel", { hasText: "Population" })
    .first()
    .screenshot({ path: join(SHOTS, "population-round4-dark.png") });
  await h.toggleTheme();
  await page.waitForTimeout(250);

  // Click-through: first cell -> lab tab -> machine viewer shows the pick.
  await page.getByTestId("pop-cell").first().click();
  await page.getByTestId("stat-generation").waitFor({ timeout: 5000 }); // lab view back
  const viewerNote = await page
    .locator("section.panel", { hasText: "Machine viewer" })
    .locator(".note")
    .first()
    .innerText();
  ok(
    "pop-click-through",
    /gen \d+ · #\d+/.test(viewerNote),
    `machine viewer note reads "${viewerNote}"`,
  );

  await page.getByTestId("stop-run").click();
  await page
    .getByTestId("stat-status")
    .filter({ hasText: "done" })
    .waitFor({ timeout: 120_000 });
  await browser.close();
});
