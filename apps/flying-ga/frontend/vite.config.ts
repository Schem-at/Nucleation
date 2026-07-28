import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { mockApiPlugin } from "./mock/api";

// The mock API is served as dev-server middleware at /api/*.
// To point the dashboard at a real backend instead, remove `mockApiPlugin()`
// and uncomment the proxy block below (real backend on :8441, same routes).
export default defineConfig({
  plugins: [react(), mockApiPlugin()],
  server: {
    port: 8440,
    strictPort: true,
    // proxy: { "/api": { target: "http://localhost:8441", changeOrigin: true } },
  },
});
