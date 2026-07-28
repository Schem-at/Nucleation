/** WebGL flight-stage verification: runs a short GA session against the vite
 * preview on :8444, waits for a champion with a flight loop, screenshots the
 * real-mesher WebGL stage mid-flight (`-webgl` suffix) plus the WebGL
 * MachineViewer, and reads mesh timing stats.
 *
 * Run: node scripts/verify-webgl.mjs [runSeconds]
 */

import { chromium } from "playwright";
import { mkdirSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const SHOTS = join(ROOT, "screenshots");
mkdirSync(SHOTS, { recursive: true });

const RUN_SECONDS = Number(process.argv[2] ?? 45);
const URL = "http://localhost:8444/";

const out = { console: [], errors: [], results: {} };
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1480, height: 1050 } });
page.on("console", (m) => {
  const t = m.text();
  if (m.type() === "error") out.console.push(`error: ${t}`);
  if (t.startsWith("[mesh]")) out.results.meshLog = t;
  if (t.startsWith("[gl]")) (out.results.glEvents ??= []).push(t);
});
page.on("pageerror", (e) => out.errors.push(String(e)));

await page.goto(URL, { waitUntil: "networkidle" });

await page.locator("#cfg-pop").fill("64");
await page.locator("#cfg-gens").fill("");
await page.getByTestId("start-run").click();
await page.getByTestId("stat-status").filter({ hasText: "running" }).waitFor({
  timeout: 90_000,
});
console.error("run started, evolving for", RUN_SECONDS, "s …");
await page.waitForTimeout(RUN_SECONDS * 1000);
await page.getByTestId("stop-run").click();
await page.getByTestId("stat-status").filter({ hasText: "done" }).waitFor({
  timeout: 180_000,
});

// The champion's flight loop re-simulates, then the WebGL stage mounts.
const stage = page.getByTestId("stage-webgl");
await stage.waitFor({ timeout: 120_000 });
await page.waitForTimeout(4000); // models meshed + a few loop cycles
await stage.screenshot({ path: join(SHOTS, "flight-stage-midflight-webgl.png") });
await page.waitForTimeout(600); // a different phase of the loop
await stage.screenshot({ path: join(SHOTS, "flight-stage-midflight2-webgl.png") });
out.results.stagePresent = true;

// MachineViewer WebGL stage (auto-selected top machine).
const gl = page.getByTestId("gl-blocks");
try {
  await gl.waitFor({ timeout: 20_000 });
  await page.locator('[data-testid="gl-blocks"][data-ready="1"]').waitFor({ timeout: 60_000 });
  await gl.screenshot({ path: join(SHOTS, "machine-viewer-webgl.png") });
  out.results.viewer = true;
} catch (e) {
  out.results.viewer = String(e).slice(0, 200);
}

await page.screenshot({ path: join(SHOTS, "flying-ga-page-webgl.png"), fullPage: true });

// Iso fallback toggle on the stage still works.
try {
  await page.getByTestId("stage-mode-toggle").click({ timeout: 5000 });
  await page.waitForTimeout(800);
  await page
    .locator(".stage-canvas canvas")
    .screenshot({ path: join(SHOTS, "flight-stage-iso-fallback.png") });
  out.results.isoFallback = true;
} catch (e) {
  out.results.isoFallback = String(e).slice(0, 200);
}

await browser.close();
console.log(JSON.stringify(out, null, 2));
