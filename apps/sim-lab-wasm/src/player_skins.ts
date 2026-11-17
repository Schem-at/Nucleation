/** Resolve saved head textures before building any chunk's texture atlas. */
export type SkinStatus = { requested: number; loaded: number; failed: string[] };
const downloads = new Map<string, Promise<Uint8Array>>();
const registered = new WeakMap<object, Set<string>>();

export function skinHash(value: string): string | null {
  try {
    const url = new URL(value);
    if (!['http:', 'https:'].includes(url.protocol) || url.hostname !== 'textures.minecraft.net'
        || url.port || url.username || url.password || url.search || url.hash) return null;
    return /^\/texture\/([a-f0-9]{1,64})$/i.exec(url.pathname)?.[1].toLowerCase() ?? null;
  } catch { return null; }
}

async function download(hash: string): Promise<Uint8Array> {
  let pending = downloads.get(hash);
  if (!pending) {
    pending = (async () => {
      // Atlas keeps a persistent local copy; standalone Sim Lab uses Mojang's
      // CORS-enabled texture CDN directly. Old saved http URLs become HTTPS.
      const params = new URLSearchParams(location.search);
      const url = params.has('archive') || params.get('local') === '1'
        ? `/api/player-skin/${hash}` : `https://textures.minecraft.net/texture/${hash}`;
      const response = await fetch(url, { signal: AbortSignal.timeout(8000) });
      if (!response.ok) throw new Error(`skin ${hash}: HTTP ${response.status}`);
      const bytes = new Uint8Array(await response.arrayBuffer());
      if (bytes.length > 256 * 1024) throw new Error('Skin exceeds size limit');
      return bytes;
    })();
    downloads.set(hash, pending);
    // A transient offline failure must not poison all subsequent reloads.
    void pending.catch(() => downloads.delete(hash));
  }
  return pending;
}

// The generated bridge is intentionally loaded at runtime.
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export async function loadPlayerSkins(eng: any, schematic: any, pack: any): Promise<SkinStatus> {
  const urls: string[] = JSON.parse(eng.ResourcePack.playerSkinUrlsJson(schematic));
  const unique = [...new Set(urls)];
  const status: SkinStatus = { requested: unique.length, loaded: 0, failed: [] };
  let done = registered.get(pack);
  if (!done) { done = new Set(); registered.set(pack, done); }
  let cursor = 0;
  await Promise.all(Array.from({ length: Math.min(4, unique.length) }, async () => {
    while (cursor < unique.length) {
      const url = unique[cursor++], hash = skinHash(url);
      try {
        if (!hash) throw new Error('Unsupported saved skin URL');
        if (!done.has(hash)) {
          pack.addPlayerSkin(hash, await download(hash));
          done.add(hash);
        }
        status.loaded++;
      } catch {
        if (hash) downloads.delete(hash); // also retry an invalid image response
        status.failed.push(url);
      }
    }
  }));
  return status;
}
