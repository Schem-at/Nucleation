import { serve, file } from "bun";
import { join } from "path";

const PORT = 3001;
const PUBLIC_DIR = import.meta.dir;
const WASM_DIR = join(import.meta.dir, "..", "pkg");

const MIME_TYPES: Record<string, string> = {
  ".html": "text/html",
  ".js": "application/javascript",
  ".mjs": "application/javascript",
  ".css": "text/css",
  ".json": "application/json",
  ".wasm": "application/wasm",
  ".png": "image/png",
  ".jpg": "image/jpeg",
  ".glb": "model/gltf-binary",
  ".gltf": "model/gltf+json",
};

function getMimeType(path: string): string {
  const ext = path.substring(path.lastIndexOf("."));
  return MIME_TYPES[ext] || "application/octet-stream";
}

serve({
  port: PORT,
  async fetch(req) {
    const url = new URL(req.url);
    let pathname = url.pathname;

    // Handle CORS for development
    const headers = {
      "Access-Control-Allow-Origin": "*",
      "Access-Control-Allow-Methods": "GET, POST, OPTIONS",
      "Access-Control-Allow-Headers": "Content-Type",
    };

    if (req.method === "OPTIONS") {
      return new Response(null, { headers });
    }

    // Serve index.html for root
    if (pathname === "/") {
      pathname = "/index.html";
    }

    // Try to serve from pkg directory (WASM files)
    if (pathname.startsWith("/pkg/")) {
      const filePath = join(WASM_DIR, pathname.replace("/pkg/", ""));
      const f = file(filePath);
      if (await f.exists()) {
        return new Response(f, {
          headers: {
            ...headers,
            "Content-Type": getMimeType(filePath),
          },
        });
      }
    }

    // Try to serve from public directory
    const filePath = join(PUBLIC_DIR, pathname);
    const f = file(filePath);
    if (await f.exists()) {
      return new Response(f, {
        headers: {
          ...headers,
          "Content-Type": getMimeType(filePath),
        },
      });
    }

    return new Response("Not Found", { status: 404, headers });
  },
});

console.log(`Nucleation Mesh Viewer running at http://localhost:${PORT}`);
console.log(`\nMake sure to build WASM with both wasm AND meshing features:`);
console.log(`  wasm-pack build --target web --features "wasm,meshing"`);
console.log(`\nThe pkg/ directory should contain nucleation.js and nucleation_bg.wasm`);
