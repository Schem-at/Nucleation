import { defineConfig } from "vite";
import { createHash } from "node:crypto";
import { readFileSync, existsSync } from "node:fs";
import path from "node:path";

// Fully static, backendless build (door-cert-wasm pattern). The wasm engine
// lives in public/engine/ (copied from dist/npm-eda by sync-engine) and is
// imported at runtime by URL so the .wasm fetch has a stable path — never
// through Vite's dependency pipeline.
//
// STALE ENGINE, TWO WAYS, TWO DEFENCES.
//
// A stable path is what makes the engine cacheable, and a cached engine is how
// a verify run ends up measuring a router that no longer exists — which is
// exactly what happened: three gate checks failed against a reason string that
// had already been deleted from the source. So:
//
//   * `__ENGINE_SHA__` is the hash of the engine ON DISK at the moment this
//     config is loaded (i.e. at build or dev-server start). The page compares it
//     with the stamp that shipped beside the engine it actually loaded, and says
//     so loudly if they differ. That catches a build whose public/engine was
//     behind, and a browser cache serving an older wasm than the server has.
//   * `/engine/*` is served `no-store` in dev AND preview. The harness drives
//     preview, so this is the one that keeps `npm run verify` honest.
const ENGINE = path.join(__dirname, "..", "..", "dist", "npm-eda", "nucleation.wasm");
const engineSha = existsSync(ENGINE)
  ? createHash("sha256").update(readFileSync(ENGINE)).digest("hex").slice(0, 16)
  : "";

/** No caching for the engine, in both servers. */
const noStoreEngine = () => {
  const mw = (req: { url?: string }, res: { setHeader(k: string, v: string): void }, next: () => void) => {
    if (req.url?.startsWith("/engine/")) res.setHeader("Cache-Control", "no-store, must-revalidate");
    next();
  };
  return {
    name: "engine-no-store",
    configureServer(server: { middlewares: { use(fn: unknown): void } }) { server.middlewares.use(mw); },
    configurePreviewServer(server: { middlewares: { use(fn: unknown): void } }) { server.middlewares.use(mw); },
  };
};

export default defineConfig({
  plugins: [noStoreEngine()],
  define: { __ENGINE_SHA__: JSON.stringify(engineSha) },
  server: { port: 8455, strictPort: true },
  preview: { port: 8455, strictPort: true },
  build: { target: "es2022" },
  optimizeDeps: { exclude: ["@yowasp/yosys"] },
});
