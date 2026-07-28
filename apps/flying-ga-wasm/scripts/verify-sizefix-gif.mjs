/** Companion to verify-sizefix.mjs: engine-b run, clean stage screenshot
 * (drawer closed) + GIF export of the looping champion.
 * Run: node scripts/verify-sizefix-gif.mjs */

import { chromium } from "playwright";
import { join } from "node:path";
import { withPreview, ok, failures, SHOTS, URL } from "./round4-lib.mjs";

await withPreview(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width: 1480, height: 1050 } });
  await page.goto(URL, { waitUntil: "networkidle" });

  await page.getByTestId("group-genome").locator("summary").click();
  await page.getByTestId("cfg-seeding").selectOption("engine-b");
  await page.locator("#cfg-pop").fill("32");
  await page.locator("#cfg-gens").fill("");
  await page.getByTestId("start-run").click();
  await page
    .getByTestId("stat-status")
    .filter({ hasText: "running" })
    .waitFor({ timeout: 90_000 });

  // Wait until the stage has a looping champion (export button enables).
  await page.waitForFunction(
    () => {
      const b = document.querySelector('[data-testid="export-gif"]');
      return b && !b.disabled;
    },
    { timeout: 120_000 },
  );
  await page.getByTestId("pause-run").click();
  // Close the config drawer so nothing overlays the stage.
  await page.getByTestId("config-drawer-toggle").click();
  await page.waitForTimeout(1200);
  await page.screenshot({ path: join(SHOTS, "stage-loop-sizefix.png") });

  try {
    const [dl] = await Promise.all([
      page.waitForEvent("download", { timeout: 90_000 }),
      page.getByTestId("export-gif").click(),
    ]);
    await dl.saveAs(join(SHOTS, "flight-sizefix.gif"));
    ok("gif-exported", true, "screenshots/flight-sizefix.gif");
  } catch (e) {
    ok("gif-exported", false, String(e).split("\n")[0]);
  }

  await page.getByTestId("config-drawer-toggle").click();
  await page.getByTestId("stop-run").click();
  await page
    .getByTestId("stat-status")
    .filter({ hasText: "done" })
    .waitFor({ timeout: 120_000 });
  await browser.close();
});

console.log(failures.length === 0 ? "GIF PASS" : `FAIL: ${failures.join("; ")}`);
process.exit(failures.length === 0 ? 0 : 1);
