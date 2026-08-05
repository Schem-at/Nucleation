import { chromium } from 'playwright';
const file = process.argv[2];
const b = await chromium.launch();
const p = await b.newPage();
const errs = [];
p.on('console', m => { if (m.type() === 'error') errs.push(m.text().slice(0,200)); });
p.on('pageerror', e => errs.push('PAGEERROR ' + String(e).slice(0,200)));
await p.goto('http://127.0.0.1:8455/', { waitUntil: 'networkidle' });
await p.setInputFiles('input[type=file]', file);
try {
  await p.waitForSelector('.status.ready', { timeout: 180000 });
} catch { console.log('status:', await p.textContent('.status')); }
console.log('status:', (await p.textContent('.status'))?.trim());
// step the sim a few times
for (let i = 0; i < 5; i++) { await p.click('button:has-text("step")'); await p.waitForTimeout(120); }
console.log('tick:', (await p.textContent('.tick'))?.trim());
await p.screenshot({ path: '/tmp/simlab.png' });
console.log('errors:', errs.slice(0,4));
await b.close();
