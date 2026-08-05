import { chromium } from 'playwright';
const b = await chromium.launch(); const p = await b.newPage();
await p.goto('http://127.0.0.1:8455/', { waitUntil: 'networkidle' });
console.log(await p.evaluate(async () => {
  const eng = await import('/engine/index.mjs');
  const s = eng.Schematic.create('probe');
  const blocks = ['minecraft:heavy_core[waterlogged=false]','minecraft:loom[facing=south]',
                  'minecraft:stonecutter[facing=north]','minecraft:stone_slab[type=bottom,waterlogged=false]',
                  'minecraft:polished_andesite','minecraft:sandstone','minecraft:end_rod[facing=up]',
                  'minecraft:lever[face=floor,facing=north,powered=false]'];
  blocks.forEach((bl,i) => s.setBlockFromString(i, 1, 0, bl));
  for (let i=0;i<blocks.length;i++) s.setBlockFromString(i, 0, 0, 'minecraft:stone');
  try {
    const sim = eng.TickSimulation.fromSchematic(s, eng.TickSettleMode.InWorld, 0,0,0, "");
    sim.step();
    return `ALL ${blocks.length} BLOCKS OK — sim built and stepped, tick ${sim.tickCount()}`;
  } catch(e) {
    let d=''; try { d = eng.TickSimulation.lastErrorDetail(); } catch {}
    return `REFUSED: ${e} — ${d}`;
  }
}));
await b.close();
