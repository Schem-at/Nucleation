import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Fully static, backendless build. The wasm engine lives in public/engine/
// (copied from dist/npm-mctick) and is imported at runtime by URL so the
// .wasm fetch has a stable path — never through Vite's dependency pipeline.
export default defineConfig({
  plugins: [react()],
  server: { port: 8433, strictPort: true },
  preview: { port: 8433, strictPort: true },
});
