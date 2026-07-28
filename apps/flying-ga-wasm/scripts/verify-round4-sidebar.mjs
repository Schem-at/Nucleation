/** Round-4 (a): sidebar redesign. At 1280x800 and 1920x1080, both themes,
 * with every group open: the rail never overflows horizontally
 * (scrollWidth <= clientWidth), fits the viewport via internal scroll, and
 * the group summaries are keyboard-toggleable. Screenshots: sidebar-round4-*.
 * Run: node scripts/verify-round4-sidebar.mjs */

import { chromium } from "playwright";
import { join } from "node:path";
import { withPreview, helpers, ok, SHOTS, URL } from "./round4-lib.mjs";

const GROUPS = ["run", "genome", "objectives", "constraints", "schedules", "advanced"];

await withPreview(async () => {
  const browser = await chromium.launch();
  for (const vp of [
    { width: 1280, height: 800 },
    { width: 1920, height: 1080 },
  ]) {
    const page = await browser.newPage({ viewport: vp });
    const h = helpers(page);
    await page.goto(URL, { waitUntil: "networkidle" });
    for (const theme of ["light", "dark"]) {
      const isDark = await page.evaluate(
        () => document.documentElement.dataset.theme === "dark",
      );
      if ((theme === "dark") !== isDark) await h.toggleTheme();
      await page.waitForTimeout(200);
      for (const g of GROUPS) await h.openGroup(g);
      const m = await page.locator(".side-col").evaluate((el) => ({
        scrollW: el.scrollWidth,
        clientW: el.clientWidth,
        rectW: el.getBoundingClientRect().width,
        rectH: el.getBoundingClientRect().height,
        scrollH: el.scrollHeight,
        clientH: el.clientHeight,
      }));
      ok(
        `rail-no-x-overflow-${vp.width}-${theme}`,
        m.scrollW <= m.clientW,
        `scrollWidth ${m.scrollW} <= clientWidth ${m.clientW}`,
      );
      ok(
        `rail-fits-viewport-${vp.width}-${theme}`,
        m.rectH <= vp.height,
        `rail height ${Math.round(m.rectH)} <= viewport ${vp.height} (internal scroll: content ${m.scrollH})`,
      );
      const docOverflow = await page.evaluate(
        () => document.documentElement.scrollWidth - window.innerWidth,
      );
      ok(
        `page-no-x-overflow-${vp.width}-${theme}`,
        docOverflow <= 0,
        `document horizontal overflow ${docOverflow}px`,
      );
      if (vp.width === 1280)
        await page
          .locator(".side-col")
          .screenshot({ path: join(SHOTS, `sidebar-round4-${theme}.png`) });
    }
    // Keyboard: focus the Genome summary and toggle it with Enter.
    const genome = page.getByTestId("group-genome");
    await genome.locator("summary").focus();
    const before = await genome.evaluate((el) => el.open);
    await page.keyboard.press("Enter");
    await page.waitForTimeout(150);
    const after = await genome.evaluate((el) => el.open);
    ok(
      `summary-keyboard-${vp.width}`,
      before !== after,
      `Enter toggled group-genome ${before} -> ${after}`,
    );
    await page.close();
  }
  await browser.close();
});
