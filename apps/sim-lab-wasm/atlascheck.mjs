import { chromium } from "playwright";
const b = await chromium.launch(); const p = await b.newPage();
await p.goto("http://localhost:8455/", { waitUntil: "networkidle" });
await p.setInputFiles("input[type=file]", process.argv[2]);
await p.waitForSelector(".status.ready", { timeout: 300000 });
console.log(await p.evaluate(async () => {
  const w = window.simlab.world, CHUNK = 16, out = [];
  const imageOf = (cx,cy,cz) => {
    const s = w.eng.Schematic.create("c");
    s.copyRegion(w.schem, cx*CHUNK,cy*CHUNK,cz*CHUNK,
      cx*CHUNK+CHUNK-1, cy*CHUNK+CHUNK-1, cz*CHUNK+CHUNK-1, cx*CHUNK,cy*CHUNK,cz*CHUNK, "[]");
    if ((s.blockCount?.() ?? 0) === 0) return null;
    const m = w.eng.MeshResult.create(s, w.pack, w.cfg);
    const bin = atob(m.glbDataB64()); const f = new Uint8Array(bin.length);
    for (let i=0;i<bin.length;i++) f[i]=bin.charCodeAt(i);
    const dv = new DataView(f.buffer); const jl = dv.getUint32(12,true);
    const j = JSON.parse(new TextDecoder().decode(f.subarray(20,20+jl)));
    const img = j.images?.[0]; if (!img) return null;
    const bv = j.bufferViews[img.bufferView];
    const binStart = 20 + jl + 8;
    const bytes = f.subarray(binStart + (bv.byteOffset||0), binStart + (bv.byteOffset||0) + bv.byteLength);
    // cheap content hash
    let h = 2166136261;
    for (let i=0;i<bytes.length;i++) { h ^= bytes[i]; h = Math.imul(h, 16777619); }
    return { len: bytes.length, hash: (h>>>0).toString(16) };
  };
  const seen = [];
  for (const k of [...w.chunks.keys()].slice(0, 6)) {
    const [cx,cy,cz] = k.split(",").map(Number);
    const r = imageOf(cx,cy,cz);
    if (r) { out.push(`chunk ${k}: atlas ${r.len} bytes hash ${r.hash}`); seen.push(r.hash); }
  }
  const same = seen.every(h => h === seen[0]);
  out.push(same ? `✅ all ${seen.length} chunks embed the IDENTICAL atlas — safe to share`
                : `❌ atlases DIFFER between chunks — sharing materials would show wrong textures`);
  return out.join("\n");
}));
await b.close();
