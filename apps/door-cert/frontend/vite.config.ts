import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { mockApiPlugin } from "./mock/plugin";

// Dev: `npm run dev` serves the app on :8430 with the mock API mounted as
// vite middleware at /api/* (no second process needed).
// Prod: `npm run build` emits a fully static dist/ — the real backend serves
// it and answers the same /api/* routes.
export default defineConfig({
  plugins: [react(), mockApiPlugin()],
  server: { port: 8430, strictPort: true },
});
