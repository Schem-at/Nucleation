import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// The wasm engine lives in public/engine/ and is imported at runtime with a
// bare dynamic import("/engine/index.mjs") (see tests/browser_bench/README.md)
// — it must NOT go through the bundler, so nothing here references it.
export default defineConfig({
  plugins: [react()],
  build: { target: "esnext" },
  worker: { format: "es" },
});
