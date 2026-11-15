import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// The wasm engine lives in public/engine/ and is imported at runtime with a
// bare dynamic import("/engine/index.mjs") (see tests/browser_bench/README.md)
// — it must NOT go through the bundler, so nothing here references it.
export default defineConfig({
  // Atlas mounts this app at /sim-lab/; standalone previews also work.
  base: "./",
  plugins: [react()],
  build: { target: "esnext" },
  worker: { format: "es" },
});
