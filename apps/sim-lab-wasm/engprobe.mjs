import { chromium } from 'playwright';
import { readFileSync } from 'fs';
const b64 = readFileSync(process.argv[2]).toString('base64');
const b = await chromium.launch(); const p = await b.newPage();
await p.goto('http://127.0.0.1:8455/', { waitUntil: 'networkidle' });
console.log((await p.evaluate(async (b64) => {
  const log = [];
  const eng = await import('/engine/index.mjs');
  const s = eng.Schematic.fromData(Uint8Array.from(atob(b64), c=>c.charCodeAt(0)));
  log.push('schematic ok, blocks ' + s.blockCount());
  log.push('has lastErrorDetail: ' + (typeof eng.TickSimulation.lastErrorDetail));
  try { const sim = eng.TickSimulation.fromSchematic(s, eng.TickSettleMode.Placement,0,0,0,"");
        log.push('SIM OK tick ' + sim.tickCount()); }
  catch(e){ log.push('sim threw: ' + e);
            try { log.push('detail: ' + eng.TickSimulation.lastErrorDetail()); }
            catch(e2){ log.push('detail unavailable: ' + String(e2).slice(0,80)); } }
  return log.join('\n');
}, b64)));
await b.close();
