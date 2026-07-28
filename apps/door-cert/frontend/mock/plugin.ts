import type { Plugin } from "vite";
import type { IncomingMessage, ServerResponse } from "node:http";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

// ---------------------------------------------------------------------------
// Mock backend for local development. Mounted as vite middleware so a single
// `npm run dev` serves both the app and /api/*. The real backend replaces
// this by answering the same routes in front of the built dist/.
//
// Routes:
//   POST /api/doors                    -> { id }
//   GET  /api/doors/:id                -> { status, step, error }
//   GET  /api/doors/:id/certificate    -> certificate JSON (fixture)
//   GET  /api/doors/:id/animation.mp4  -> fixture video (range-aware)
// ---------------------------------------------------------------------------

const here = path.dirname(fileURLToPath(import.meta.url));
const FIXTURE_JSON = path.join(here, "fixtures", "certificate.json");
const FIXTURE_MP4 = path.join(here, "fixtures", "door_6x6_cycle.mp4");

const STEPS = ["parsing", "simulating", "measuring", "rendering"] as const;
const STEP_MS = 1400; // each processing step lasts this long

/** id -> upload timestamp. Unknown ids are treated as long-finished. */
const uploads = new Map<string, number>();

function json(res: ServerResponse, body: unknown, code = 200) {
  res.statusCode = code;
  res.setHeader("Content-Type", "application/json");
  res.end(JSON.stringify(body));
}

function statusFor(id: string) {
  const t0 = uploads.get(id);
  if (t0 === undefined) return { status: "done", step: null, error: null };
  const idx = Math.floor((Date.now() - t0) / STEP_MS);
  if (idx >= STEPS.length) return { status: "done", step: null, error: null };
  return { status: "processing", step: STEPS[idx], error: null };
}

function serveVideo(req: IncomingMessage, res: ServerResponse) {
  const stat = fs.statSync(FIXTURE_MP4);
  const range = req.headers.range;
  res.setHeader("Content-Type", "video/mp4");
  res.setHeader("Accept-Ranges", "bytes");
  if (range) {
    const m = /bytes=(\d*)-(\d*)/.exec(range);
    const start = m && m[1] ? parseInt(m[1], 10) : 0;
    const end = m && m[2] ? parseInt(m[2], 10) : stat.size - 1;
    res.statusCode = 206;
    res.setHeader("Content-Range", `bytes ${start}-${end}/${stat.size}`);
    res.setHeader("Content-Length", end - start + 1);
    fs.createReadStream(FIXTURE_MP4, { start, end }).pipe(res);
  } else {
    res.setHeader("Content-Length", stat.size);
    fs.createReadStream(FIXTURE_MP4).pipe(res);
  }
}

export function mockApiPlugin(): Plugin {
  return {
    name: "door-cert-mock-api",
    configureServer(server) {
      server.middlewares.use((req, res, next) => {
        const url = (req.url ?? "").split("?")[0];
        if (!url.startsWith("/api/")) return next();

        if (req.method === "POST" && url === "/api/doors") {
          // Consume (and discard) the multipart body, then hand back an id.
          req.on("data", () => {});
          req.on("end", () => {
            const id = Math.random().toString(36).slice(2, 8);
            uploads.set(id, Date.now());
            json(res, { id });
          });
          return;
        }

        const anim = /^\/api\/doors\/([^/]+)\/animation\.mp4$/.exec(url);
        if (req.method === "GET" && anim) return serveVideo(req, res);

        const cert = /^\/api\/doors\/([^/]+)\/certificate$/.exec(url);
        if (req.method === "GET" && cert) {
          const body = JSON.parse(fs.readFileSync(FIXTURE_JSON, "utf8"));
          body.animation_url = `/api/doors/${cert[1]}/animation.mp4`;
          return json(res, body);
        }

        const door = /^\/api\/doors\/([^/]+)$/.exec(url);
        if (req.method === "GET" && door) return json(res, statusFor(door[1]));

        json(res, { error: "not found" }, 404);
      });
    },
  };
}
