import { defineConfig } from "vite";

// Fully static, backendless build (door-cert-wasm pattern). The wasm engine
// lives in public/engine/ (copied from dist/npm-eda by sync-engine) and is
// imported at runtime by URL so the .wasm fetch has a stable path — never
// through Vite's dependency pipeline.
export default defineConfig({
  server: { port: 8455, strictPort: true },
  preview: { port: 8455, strictPort: true },
  build: { target: "es2022" },
  optimizeDeps: { exclude: ["@yowasp/yosys"] },
});
