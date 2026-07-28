# Door Certification Bureau — frontend

React + Vite + TypeScript frontend for the Minecraft piston-door certificate
platform. Upload a schematic, get a shareable performance certificate:
animation video, measured open/close times, per-tick activity trace, bill of
materials, and a change-footprint heatmap.

## Run

```sh
npm install
npm run dev        # http://localhost:8430
```

`npm run dev` starts vite on port 8430 with a **mock backend mounted as vite
middleware** (`mock/plugin.ts`) — no second process. It answers:

- `POST /api/doors` (multipart `file`) → `{ id }`
- `GET /api/doors/:id` → processing status (steps advance ~1.4 s each)
- `GET /api/doors/:id/certificate` → fixture JSON (`mock/fixtures/certificate.json`)
- `GET /api/doors/:id/animation.mp4` → fixture video (range-aware)

Any id works for direct links, e.g. `http://localhost:8430/door/abc123`.

## Build

```sh
npm run build      # typecheck + emit static dist/
```

`dist/` is fully static. The real backend serves it and answers the same
`/api/*` routes (plus SPA-fallback of unknown paths to `index.html`, so
`/door/:id` deep links work). The mock exists only inside the dev server.

## Fixtures

`npm run fixtures` regenerates `mock/fixtures/certificate.json`
deterministically (90 ticks, lever on at t=10, off at t=60).

## Theming

Light/dark follows the OS, with a manual toggle (persisted). Screenshot
tooling can force a theme with `?theme=dark` / `?theme=light`.
