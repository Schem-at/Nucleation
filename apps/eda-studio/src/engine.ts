/** Runtime loader for the wasm engine (door-cert-wasm pattern).
 *
 * The engine lives in public/engine/ (synced from dist/npm-eda) and is
 * imported by URL at runtime so the .wasm fetch has a stable path and never
 * runs through Vite's dependency pipeline. The design veneer rides along in
 * engine/veneer/design.mjs and is bound to the loaded core with veneer().
 */

export type Core = any;
export interface VeneerSurface {
  Design: any;
  Executor: any;
  Flat: any;
  Bus: any;
  CheckReport: any;
  DesignCheckError: any;
  Gate: (anchor: number[], step: number[], name?: string | null) => any;
  Style: (opts?: Record<string, unknown>) => Record<string, string>;
}

let loaded: Promise<{ core: Core; d: VeneerSurface }> | null = null;

declare const __ENGINE_SHA__: string;

/** WHICH ENGINE IS ACTUALLY RUNNING.
 *
 *  `expected` is the hash of `dist/npm-eda/nucleation.wasm` at the moment the
 *  app was built (baked in by vite.config.ts). `served` is the hash recorded in
 *  `engine/BUILD.json` beside the wasm the page just loaded (written by
 *  `sync-engine`). They agree unless something is stale — and a stale engine is
 *  not a harmless performance detail: it once cost three red checks blamed on
 *  the wrong lane, against a router reason string that had already been deleted.
 *  So the mismatch is REPORTED, not tolerated. */
export interface EngineStamp {
  ok: boolean;
  expected: string;
  served: string;
  bytes: number;
  /** The `content-length` the server gave for the wasm, which is how a cached
   *  response that disagrees with its own stamp gets caught. */
  fetchedBytes: number | null;
  builtAt: string | null;
  problem: string | null;
}

let stamp: EngineStamp = {
  ok: false, expected: "", served: "", bytes: 0, fetchedBytes: null,
  builtAt: null, problem: "the engine has not been loaded yet",
};

export function engineStamp(): EngineStamp {
  return { ...stamp };
}

async function checkStamp(base: string): Promise<EngineStamp> {
  const expected = typeof __ENGINE_SHA__ === "string" ? __ENGINE_SHA__ : "";
  const out: EngineStamp = {
    ok: false, expected, served: "", bytes: 0, fetchedBytes: null, builtAt: null, problem: null,
  };
  try {
    const res = await fetch(`${base}BUILD.json`, { cache: "no-store" });
    if (!res.ok) throw new Error(`BUILD.json ${res.status}`);
    const j = await res.json() as { sha256?: string; bytes?: number; mtime?: string };
    out.served = j.sha256 ?? "";
    out.bytes = j.bytes ?? 0;
    out.builtAt = j.mtime ?? null;
    const head = await fetch(`${base}nucleation.wasm`, { method: "HEAD", cache: "no-store" });
    const len = head.headers.get("content-length");
    out.fetchedBytes = len ? Number(len) : null;
  } catch (err) {
    out.problem = `could not read the engine stamp (${err})`;
    return out;
  }
  if (expected && out.served && expected !== out.served) {
    out.problem = `STALE ENGINE: the app was built against ${expected} but loaded ${out.served}` +
      ` (built ${out.builtAt}). Run \`npm run sync-engine && npm run build\`.`;
    return out;
  }
  if (out.fetchedBytes != null && out.bytes && out.fetchedBytes !== out.bytes) {
    out.problem = `STALE ENGINE: the served wasm is ${out.fetchedBytes} bytes but its stamp says` +
      ` ${out.bytes} — a cache is answering. Hard-reload, or check Cache-Control on /engine/.`;
    return out;
  }
  out.ok = true;
  return out;
}

export function loadEngine(): Promise<{ core: Core; d: VeneerSurface }> {
  loaded ??= (async () => {
    const base = new URL("engine/", document.baseURI).href;
    const core = await import(/* @vite-ignore */ `${base}index.mjs`);
    const { veneer } = await import(/* @vite-ignore */ `${base}veneer/design.mjs`);
    stamp = await checkStamp(base);
    if (!stamp.ok) console.error(`engine: ${stamp.problem}`);
    return { core, d: veneer(core) };
  })();
  return loaded;
}
