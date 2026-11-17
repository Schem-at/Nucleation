// Exercise local-file transfer and controls against the served production WASM.
import { createRequire } from 'node:module';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
const { chromium } = createRequire(new URL('../sim-lab-wasm/package.json', import.meta.url))('playwright');
const base = process.argv[2] || 'http://127.0.0.1:8767';
const output = process.argv[3] || '/tmp/atlas-drop-live';
const fixture = process.argv[4] || fileURLToPath(new URL('../../examples/schematic_builder_assets/generated/d_latch.litematic', import.meta.url));
const bytes = await readFile(fixture);
await mkdir(output, { recursive: true });
const browser = await chromium.launch({ headless: true,
  args: ['--use-angle=swiftshader', '--enable-unsafe-swiftshader'],
  ...(process.env.CHROME_PATH ? { executablePath: process.env.CHROME_PATH } : {}) });
const checks = [], errors = [], uploads = [], fileFetches = [];
const assert = (ok, label) => { if (!ok) throw new Error(label); checks.push(label); };
const page = await browser.newPage({ viewport: { width: 1440, height: 1000 } });
page.on('pageerror', error => errors.push(error.message));
page.on('request', request => {
  if (!['GET', 'HEAD'].includes(request.method())) uploads.push(request.url());
  if (request.url().includes('/api/file/')) fileFetches.push(request.url());
});
const ready = () => page.locator('#local-status[data-state=ready]').waitFor({ timeout: 90000 });
const frame = () => page.frameLocator('#local-stage iframe');
async function drop(files) {
  const data = await page.evaluateHandle(files => {
    const transfer = new DataTransfer();
    for (const file of files) transfer.items.add(new File([new Uint8Array(file.bytes)], file.name));
    return transfer;
  }, files);
  await page.locator('#local-drop').dispatchEvent('dragenter', { dataTransfer: data });
  await page.locator('#local-drop').dispatchEvent('dragover', { dataTransfer: data });
  await page.locator('#local-drop').dispatchEvent('drop', { dataTransfer: data });
  await data.dispose();
}
async function choose(file = fixture) {
  const choosing = page.waitForEvent('filechooser');
  await page.locator('#local-replace').click();
  await (await choosing).setFiles(file);
  await ready();
}
try {
  await page.goto(base);
  await page.locator('#results[aria-busy=false]').waitFor();
  await page.screenshot({ path: path.join(output, 'drop-zone-desktop.png') });
  await drop([{ name: path.basename(fixture), bytes: [...bytes] }]);
  await ready();
  await page.waitForFunction(() => document.querySelector('#local-tick').textContent === '0');
  assert(await page.locator('#local-live-title').innerText() === path.basename(fixture), 'dropped filename displayed');
  assert(await frame().locator('canvas').count() === 1, 'real renderer canvas attached');
  assert(await frame().locator('.status.ready').isVisible(), 'dropped schematic parsed, meshed and simulated');
  const live = page.frames().find(f => f.url().includes('local=1'));
  const loaded = await live.evaluate(() => ({ dims: window.simlab.world.dims, tick: Number(window.simlab.world.sim.tickCount()) }));
  assert(loaded.dims.every(d => d > 0) && loaded.tick === 0, 'nonempty build starts paused at tick zero');
  if (/hipster/i.test(fixture)) assert(loaded.dims.join(',') === '13,8,18', 'hipster dimensions match the supplied original');
  await frame().getByRole('button', { name: 'step', exact: true }).click();
  await page.waitForFunction(() => document.querySelector('#local-tick').textContent === '1');
  checks.push('step advances real simulation and parent telemetry');
  await frame().getByRole('button', { name: 'run', exact: true }).click();
  await page.waitForFunction(() => Number(document.querySelector('#local-tick').textContent) > 2);
  await frame().getByRole('button', { name: 'pause', exact: true }).click();
  await page.waitForFunction(() => document.querySelector('#local-run-state').textContent === 'Paused');
  const pausedTick = await live.evaluate(() => Number(window.simlab.world.sim.tickCount()));
  await page.waitForTimeout(500);
  assert(await live.evaluate(() => Number(window.simlab.world.sim.tickCount())) === pausedTick, 'pause holds actual engine tick');
  await frame().getByRole('button', { name: 'reset', exact: true }).click();
  await page.waitForFunction(() => document.querySelector('#local-tick').textContent === '0');
  checks.push('reset reloads the dropped original');
  await page.screenshot({ path: path.join(output, 'live-desktop.png') });
  const invalidReplacement = page.waitForEvent('filechooser');
  await page.locator('#local-replace').click();
  await (await invalidReplacement).setFiles({ name: 'empty.snbt', mimeType: 'text/plain', buffer: Buffer.alloc(0) });
  await page.waitForTimeout(600); // allow several telemetry messages
  assert((await page.locator('#local-status').innerText()).includes('empty') && !live.isDetached(), 'invalid replacement remains visible and preserves the current session');
  await choose();
  assert(live.isDetached(), 'choosing another file removes the old simulation');
  await choose();
  checks.push('same file can be chosen repeatedly');
  await page.setViewportSize({ width: 390, height: 844 });
  assert(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), 'mobile page has no horizontal overflow');
  const canvas = await frame().locator('canvas').boundingBox();
  assert(canvas.width >= 350 && canvas.height >= 260 && canvas.y + canvas.height < 844, 'mobile simulation has a visible usable canvas');
  await page.screenshot({ path: path.join(output, 'live-mobile.png') });
  await page.locator('#local-close').click();
  await page.locator('#local-stage iframe').waitFor({ state: 'detached' });
  assert(await page.locator('#local-stage iframe').count() === 0, 'closing removes the running iframe');
  assert(await page.locator('#local-drop').evaluate(el => el === document.activeElement), 'close restores keyboard focus');
  await page.locator('#local-drop').scrollIntoViewIfNeeded();
  await page.screenshot({ path: path.join(output, 'drop-zone-mobile.png') });
  await page.evaluate(() => {
    const dataTransfer = new DataTransfer();
    dataTransfer.items.add(new File([new ArrayBuffer(128 * 1024 * 1024 + 1)], 'oversized.litematic'));
    document.querySelector('#local-drop').dispatchEvent(new DragEvent('drop', { bubbles: true, cancelable: true, dataTransfer }));
  });
  assert((await page.locator('#local-file-error').innerText()).includes('larger than 128')
    && await page.locator('#local-stage iframe').count() === 0, 'oversized file rejected before creating a simulator');
  await drop([{ name: 'empty.snbt', bytes: [] }]);
  assert((await page.locator('#local-file-error').innerText()).includes('empty'), 'empty file rejected before loading');
  await drop([{ name: 'a.snbt', bytes: [1] }, { name: 'b.snbt', bytes: [1] }]);
  assert((await page.locator('#local-file-error').innerText()).includes('one schematic'), 'multiple files rejected clearly');
  // Keyboard chooser exercises the non-drag path and an actual malformed file.
  await page.locator('#local-drop').focus();
  const choosing = page.waitForEvent('filechooser');
  await page.keyboard.press('Enter');
  await (await choosing).setFiles({ name: 'invalid.litematic', mimeType: 'application/octet-stream', buffer: Buffer.from('this is not a schematic') });
  await page.locator('#local-status[data-state=error]').waitFor({ timeout: 90000 });
  assert((await page.locator('#local-status').innerText()).length > 10, 'malformed file reports a readable engine error');
  await choose();
  checks.push('valid file recovers after malformed input');
  await page.locator('#local-close').focus();
  await page.keyboard.press('Escape');
  await page.locator('#local-live').waitFor({ state: 'hidden' });
  await page.locator('#local-stage iframe').waitFor({ state: 'detached' });
  assert(await page.locator('#local-stage iframe').count() === 0, 'Escape disposes the local session');
  assert(uploads.length === 0 && fileFetches.length === 0, 'schematic bytes remain local: no upload or archive-file fetch');
  assert(errors.length === 0, 'no uncaught browser errors');
  const result = { status: 'pass', fixture: path.basename(fixture), checks, loaded, errors, uploads, fileFetches,
    browser: browser.version(), scope: 'Local file transfer, live WASM controls and responsive UI; not new vanilla parity evidence.' };
  await writeFile(path.join(output, 'result.json'), JSON.stringify(result, null, 2) + '\n');
  console.log(JSON.stringify(result));
} catch (error) {
  await page.screenshot({ path: path.join(output, 'failure.png') }).catch(() => {});
  await writeFile(path.join(output, 'failure.json'), JSON.stringify({ error: String(error), checks, errors, uploads, fileFetches }, null, 2));
  throw error;
} finally { await browser.close(); }
