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

export function loadEngine(): Promise<{ core: Core; d: VeneerSurface }> {
  loaded ??= (async () => {
    const base = new URL("engine/", document.baseURI).href;
    const core = await import(/* @vite-ignore */ `${base}index.mjs`);
    const { veneer } = await import(/* @vite-ignore */ `${base}veneer/design.mjs`);
    return { core, d: veneer(core) };
  })();
  return loaded;
}
