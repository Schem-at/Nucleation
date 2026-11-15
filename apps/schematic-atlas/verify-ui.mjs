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
  await page.setViewportSize({ width: 1440, height: 1000 });
  await page.locator('#validation-filter').selectOption('all');
  await page.goto(`${base}/#collection=plot-verified`);
  await page.waitForFunction(() => document.querySelector('#library-title').textContent === 'Verified plot doors');
  assert(await page.locator('#results .schematic-card').count() === 1, 'strict door collection contains the independently verified door');
  assert(await page.locator('#collection-download').getAttribute('href') === '/api/door-bundle/verified-doors', 'verified collection download points to its local bundle');
  await page.goto(`${base}/#collection=plot-interactive`);
  await page.waitForFunction(() => document.querySelector('#library-title').textContent === 'Interactive candidates');
  assert((await page.locator('#collection-note').innerText()).includes('not established'), 'motion candidates retain the incomplete-functionality label');
  assert(await page.locator('#collection-download').getAttribute('href') === '/api/door-bundle/interactive-door-candidates', 'candidate download stays separate from verified doors');
  await page.locator('#results .schematic-card').first().click();
  await page.getByText('Extracted run protocol', { exact: true }).waitFor();
  assert(await page.getByRole('link', { name: 'Download run protocol ↓', exact: true }).count() === 1, 'individual candidate exposes its recorded control protocol');
  await page.locator('#close-detail').click();
  await page.goto(`${base}/#collection=all`);
  await page.locator('#search').fill('hipster');
  await page.waitForFunction(() => document.querySelectorAll('#results .schematic-card').length === 1);
  assert((await page.locator('#results').innerText()).includes('SoulBanished'), 'supplied hipster is indexed and searchable');
  await page.locator('#results .schematic-card').click();
  await page.locator('#detail[open]').waitFor();
  await page.waitForFunction(() => document.querySelector('#detail-title').textContent.includes('Hipster'));
  await page.locator('#detail-loading').waitFor({ state: 'hidden' });
  await page.waitForFunction(() => {
    const image = document.querySelector('.overview-art img');
    return image?.src.includes('a16c594465e5664c1a29c4b87b66745916dc99b377d68be62a960847674cd8a0')
      && image.complete && image.naturalWidth === 768;
  });
  assert(page.url().endsWith('a16c594465e5664c1a29c4b87b66745916dc99b377d68be62a960847674cd8a0'), 'search opens the exact supplied hipster bytes');
  assert((await page.locator('#property-content').innerText()).includes('2,874,240'), 'hipster reference scope is visible');
  await page.screenshot({ path: path.join(output, 'hipster-desktop.png') });
  assert(errors.length === 0, 'no uncaught browser errors');
  const report = { status: 'pass', browser: browser.version(), base, checks, errors,
    scope: 'Archive UI and evidence association; simulation correctness is checked separately.' };
  await writeFile(path.join(output, 'result.json'), JSON.stringify(report, null, 2)+'\n');
  console.log(JSON.stringify(report));
} finally { await browser.close(); }
