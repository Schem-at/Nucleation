/** Shared harness for the round-4 verification scripts: spawns `vite
 * preview --host --port 8444 --strictPort` as a child of the (foreground)
 * script, waits for it, and always kills it on the way out. */

import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

export const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
export const SHOTS = join(ROOT, "screenshots");
mkdirSync(SHOTS, { recursive: true });
export const URL = "http://localhost:8444/";

export const failures = [];
export const ok = (name, cond, detail) => {
  if (!cond) failures.push(`${name}: ${detail}`);
  console.log(`${cond ? "PASS" : "FAIL"} ${name} — ${detail}`);
};

export async function withPreview(fn) {
  const server = spawn(
    "npx",
    ["vite", "preview", "--host", "--port", "8444", "--strictPort"],
    { cwd: ROOT, stdio: "ignore" },
  );
  const kill = () => {
    try {
      server.kill("SIGTERM");
    } catch {
      /* already gone */
    }
  };
  process.on("exit", kill);
  const t0 = Date.now();
  let up = false;
  while (Date.now() - t0 < 30_000 && !up) {
    try {
      const r = await fetch(URL);
      up = r.ok;
    } catch {
      await new Promise((r) => setTimeout(r, 300));
    }
  }
  if (!up) {
    kill();
    throw new Error("preview server never came up on :8444");
  }
  try {
    await fn();
  } finally {
    kill();
  }
  if (failures.length > 0) {
    console.log(`\n${failures.length} FAILURE(S):\n${failures.join("\n")}`);
    process.exitCode = 1;
  } else {
    console.log("\nALL PASS");
  }
}

export const helpers = (page) => ({
  gen: async () =>
    parseInt(
      (await page.getByTestId("stat-generation").innerText()).split("\n")[0],
      10,
    ) || 0,
  waitGen: async function (target, timeoutMs) {
    const t0 = Date.now();
    while (Date.now() - t0 < timeoutMs) {
      const g = await this.gen();
      if (g >= target) return g;
      await page.waitForTimeout(1500);
    }
    return await this.gen();
  },
  toggleTheme: () =>
    page.getByRole("button", { name: "Toggle color theme" }).click(),
  eventsText: async () => {
    const feed = page.getByTestId("events-feed");
    return (await feed.count()) > 0 ? await feed.innerText() : "";
  },
  openGroup: async (id) => {
    const d = page.getByTestId(`group-${id}`);
    if (!(await d.evaluate((el) => el.open)))
      await d.locator("summary").click();
    await page.waitForTimeout(120);
  },
});
