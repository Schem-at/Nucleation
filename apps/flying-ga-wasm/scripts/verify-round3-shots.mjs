/** Round-3 screenshot addendum: camera-drag stability (viewer + follow chip)
 * and weighted-mode re-ranking before/after. Run: node scripts/verify-round3-shots.mjs */
import { chromium } from "playwright";
import { mkdirSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const SHOTS = join(ROOT, "screenshots");
mkdirSync(SHOTS, { recursive: true });

const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1480, height: 1050 } });
await page.goto("http://localhost:8444/", { waitUntil: "networkidle" });
const toggleTheme = () =>
  page.getByRole("button", { name: "Toggle color theme" }).click();

// Scalar engine-b run with speed + size (weights matter in scalar mode).
await page.getByTestId("cfg-seeding").selectOption("engine-b");
await page.getByTestId("obj-size").check();
await page.locator("#cfg-pop").fill("64");
await page.locator("#cfg-gens").fill("");
await page.getByTestId("start-run").click();
await page
  .locator('[data-testid="gl-blocks"][data-ready="1"]')
  .waitFor({ timeout: 120_000 });
await page.waitForTimeout(8000); // let the board fill out

// Camera drag + follow chip.
const canvas = page.locator('[data-testid="gl-blocks"] canvas');
const bb = await canvas.boundingBox();
await page.mouse.move(bb.x + bb.width / 2, bb.y + bb.height / 2);
await page.mouse.down();
for (let i = 1; i <= 8; i++)
  await page.mouse.move(bb.x + bb.width / 2 + i * 10, bb.y + bb.height / 2 + i * 4, { steps: 2 });
await page.mouse.up();
await page.getByTestId("viewer-follow-chip").waitFor({ timeout: 5000 });
const viewerPanel = page
  .locator("section.panel", { hasText: "Machine viewer" })
  .first();
await viewerPanel.screenshot({ path: join(SHOTS, "camera-drag-round3-light.png") });
await toggleTheme();
await page.waitForTimeout(300);
await viewerPanel.screenshot({ path: join(SHOTS, "camera-drag-round3-dark.png") });
await toggleTheme();
await page.waitForTimeout(300);

// Weighted re-rank: before / after size weight 1 → 8.
const lbPanel = page.locator("section.panel", { hasText: "Leaderboard" }).first();
await lbPanel.screenshot({ path: join(SHOTS, "weighted-rerank-before-round3-light.png") });
const before = await page.$$eval(".lb-table tbody tr td.name", (t) =>
  t.map((x) => x.textContent.trim()),
);
await page.getByTestId("weight-size").fill("");
await page.getByTestId("weight-size").fill("8");
await page.waitForTimeout(6000); // debounce + ≥1 generation boundary
await lbPanel.screenshot({ path: join(SHOTS, "weighted-rerank-after-round3-light.png") });
await toggleTheme();
await page.waitForTimeout(300);
await lbPanel.screenshot({ path: join(SHOTS, "weighted-rerank-after-round3-dark.png") });
await toggleTheme();
const after = await page.$$eval(".lb-table tbody tr td.name", (t) =>
  t.map((x) => x.textContent.trim()),
);
console.log("order before:", before.join(", "));
console.log("order after :", after.join(", "));
console.log("changed:", JSON.stringify(before) !== JSON.stringify(after));

await page.getByTestId("stop-run").click();
await page
  .getByTestId("stat-status")
  .filter({ hasText: "done" })
  .waitFor({ timeout: 120_000 });
await browser.close();
console.log("SHOTS OK");
