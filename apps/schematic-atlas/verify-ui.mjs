// Isolated browser check; never operates the user's browser profile.
import { createRequire } from 'node:module';
import { mkdir, writeFile } from 'node:fs/promises';
import path from 'node:path';
const { chromium } = createRequire(new URL('../sim-lab-wasm/package.json', import.meta.url))('playwright');
const base = process.argv[2] || 'http://127.0.0.1:8767';
const output = process.argv[3] || '/tmp/circuit-atlas-ui';
await mkdir(output, { recursive: true });
const browser = await chromium.launch({ headless: true,
  ...(process.env.CHROME_PATH ? { executablePath: process.env.CHROME_PATH } : {}) });
const errors = [], checks = [];
const assert = (value, message) => { if (!value) throw new Error(message); checks.push(message); };
try {
  const page = await browser.newPage({ viewport: { width: 1440, height: 1000 } });
  page.on('pageerror', e => errors.push(e.message));
  const adder = '3b4e13cd09dc8ad0b5770370d6f416e1d8140e2eec416c207ba76d8a79af46ca';
  await page.goto(`${base}/#entry=${adder}`);
  await page.getByText('Executable build adapter', { exact: true }).waitFor();
  const text = await page.locator('#property-content').innerText();
  assert(text.includes('a: u8') && text.includes('b: u8') && text.includes('cin: bool'), 'typed adder inputs visible');
  assert(text.includes('sum: u8') && text.includes('cout: bool'), 'typed adder outputs visible');
  assert(text.includes('functional_pass') && text.includes('328'), 'recorded adapter evidence visible');
  assert(await page.getByRole('link', { name: 'Runner code', exact: true }).count() === 1, 'runner source attached');
  await page.waitForFunction(() => {
    const image = document.querySelector('.overview-art img');
    return image?.complete && image.naturalWidth > 0 && getComputedStyle(image).opacity !== '0';
  });
  checks.push('real textured thumbnail visible');
  await page.screenshot({ path: path.join(output, 'adapter-desktop.png') });
  await page.setViewportSize({ width: 390, height: 844 });
  assert(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), 'mobile layout stays within viewport');
  await page.screenshot({ path: path.join(output, 'adapter-mobile.png') });
  await page.locator('#close-detail').click();
  await page.goto(`${base}/#collection=plot-doors`);
  await page.locator('#validation-filter').selectOption('smoke');
  await page.waitForFunction(() => document.querySelectorAll('#results .schematic-card').length > 0);
  assert(await page.locator('#results .chip').evaluateAll(nodes => nodes.every(n => n.textContent === 'Simulation checked')), 'smoke checks have separate evidence labels');
  await page.locator('#validation-filter').selectOption('vanilla');
  await page.waitForFunction(() => document.querySelectorAll('#results .schematic-card').length === 1);
  assert(await page.locator('#results .chip').innerText() === 'Vanilla matched', 'only admitted plot door appears as vanilla matched');
  assert(errors.length === 0, 'no uncaught browser errors');
  const report = { status: 'pass', browser: browser.version(), base, checks, errors,
    scope: 'Archive UI and evidence association; simulation correctness is checked separately.' };
  await writeFile(path.join(output, 'result.json'), JSON.stringify(report, null, 2)+'\n');
  console.log(JSON.stringify(report));
} finally { await browser.close(); }
