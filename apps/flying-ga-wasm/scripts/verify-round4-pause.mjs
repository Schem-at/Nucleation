/** Round-4 (c): pause/resume. Pause at ~gen 40 — the generation counter
 * freezes ≥ 5 s while the UI stays interactive; max-blocks is adjusted
 * WHILE paused; resume continues the same run and the change is logged.
 * Screenshots: paused-round4-{light,dark}.
 * Run: node scripts/verify-round4-pause.mjs */

import { chromium } from "playwright";
import { join } from "node:path";
import { withPreview, helpers, ok, SHOTS, URL } from "./round4-lib.mjs";

await withPreview(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width: 1480, height: 1050 } });
  const h = helpers(page);
  await page.goto(URL, { waitUntil: "networkidle" });

  await page.locator("#cfg-pop").fill("64");
  await page.getByTestId("start-run").click();
  await page
    .getByTestId("stat-status")
    .filter({ hasText: "running" })
    .waitFor({ timeout: 90_000 });
  await h.waitGen(40, 300_000);

  await page.getByTestId("pause-run").click();
  await page
    .getByTestId("stat-status")
    .filter({ hasText: "paused" })
    .waitFor({ timeout: 30_000 });
  const g1 = await h.gen();
  ok("paused-near-40", g1 >= 40, `paused at gen ${g1}`);

  // Counter must hold ≥ 5 s while the page stays interactive.
  await page.waitForTimeout(5500);
  const g2 = await h.gen();
  ok("gen-frozen-5s", g1 === g2, `gen ${g1} -> ${g2} across 5.5 s paused`);
  const evPause = await h.eventsText();
  ok(
    "pause-event",
    /paused @ gen \d+/.test(evPause),
    `events feed logs "paused @ gen N"`,
  );

  // Interactivity while paused: the population tab renders a stable
  // snapshot with the paused flag.
  await page.getByTestId("tab-population").click();
  await page.getByTestId("pop-paused-flag").waitFor({ timeout: 5000 });
  ok("interactive-while-paused", true, "population tab opened + paused flag shown");
  await page.screenshot({ path: join(SHOTS, "paused-round4-light.png"), fullPage: false });
  await h.toggleTheme();
  await page.waitForTimeout(250);
  await page.screenshot({ path: join(SHOTS, "paused-round4-dark.png"), fullPage: false });
  await h.toggleTheme();
  await page.getByRole("button", { name: "Lab" }).click();

  // Live constraint change WHILE paused: max blocks 14 -> 9 by keyboard.
  await h.openGroup("constraints");
  const maxb = page.getByTestId("cfg-maxb");
  await maxb.focus();
  for (let i = 0; i < 5; i++) await page.keyboard.press("ArrowLeft");
  const newMax = await maxb.inputValue();
  ok("maxblocks-adjustable-paused", newMax === "9", `max blocks slider now ${newMax} while paused`);
  await page.waitForTimeout(800); // let the reconfigure debounce queue it
  const gStill = await h.gen();
  ok("still-frozen-after-edit", gStill === g1, `gen still ${gStill} after editing while paused`);

  await page.getByTestId("resume-run").click();
  await page
    .getByTestId("stat-status")
    .filter({ hasText: "running" })
    .waitFor({ timeout: 30_000 });
  await page.waitForTimeout(5000); // a few generations
  const g3 = await h.gen();
  ok("resumed-continues", g3 > g1, `gen advanced ${g1} -> ${g3} after resume`);
  const ev = await h.eventsText();
  ok("resume-event", /resumed @ gen \d+/.test(ev), `events feed logs "resumed @ gen N"`);
  ok(
    "constraint-change-logged",
    ev.includes("max blocks → 9"),
    `events feed logs the paused max-blocks change (constraints changed: max blocks → 9)`,
  );

  await page.getByTestId("stop-run").click();
  await page
    .getByTestId("stat-status")
    .filter({ hasText: "done" })
    .waitFor({ timeout: 120_000 });
  await browser.close();
});
