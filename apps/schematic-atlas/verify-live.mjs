// Exercise the served build and real WASM session in an isolated browser.
import { createRequire } from 'node:module';
import { mkdir, writeFile } from 'node:fs/promises';
import path from 'node:path';
const { chromium } = createRequire(new URL('../sim-lab-wasm/package.json', import.meta.url))('playwright');
const base = process.argv[2] || 'http://127.0.0.1:8767';
const output = process.argv[3] || '/tmp/atlas-live-check';
await mkdir(output, { recursive: true });
const browser = await chromium.launch({ headless: true, args: ['--use-angle=swiftshader', '--enable-unsafe-swiftshader'],
  ...(process.env.CHROME_PATH ? { executablePath: process.env.CHROME_PATH } : {}) });
const checks = [], errors = [], failures = [];
const assert = (ok, label) => { if (!ok) throw new Error(label); checks.push(label); };
try {
  const page = await browser.newPage({ viewport: { width: 1440, height: 1000 } });
  page.on('pageerror', error => errors.push(error.message));
  page.on('response', response => { if (response.status() >= 400 && response.url().startsWith(base)) failures.push(`${response.status()} ${response.url()}`); });
  // Verify the actual built HTML, not an assumed asset filename or alias.
  const response = await page.request.get(`${base}/sim-lab/`);
  const html = await response.text();
  for (const [, asset] of html.matchAll(/(?:src|href)="([^\"]+\.(?:js|css))"/g)) {
    const url = new URL(asset, `${base}/sim-lab/`).href;
    const resource = await page.request.get(url);
    assert(resource.ok(), `served asset ${new URL(url).pathname}`);
    assert(asset.endsWith('.css') ? resource.headers()['content-type'].startsWith('text/css') : /javascript/.test(resource.headers()['content-type']), 'correct asset MIME type');
  }
  const latch = '0663302dfc40ac8c33055d5a43aefe834be4290bdda9baaec6f2ccab238cad54';
  await page.goto(`${base}/#entry=${latch}`);
  await page.locator('#launch-live:not([disabled])').waitFor();
  await page.getByRole('tab', { name: 'Live simulation', exact: true }).click();
  const iframe = page.frameLocator('#live-stage iframe');
  await iframe.locator('.status.ready').waitFor({ timeout: 90000 });
  await page.waitForFunction(() => document.querySelector('#live-tick').textContent === '0');
  assert(await page.locator('#detail').evaluate(el => el.classList.contains('details-hidden')), 'live launch expands the canvas');
  const viewport = await page.locator('.viewer-column .viewport').boundingBox();
  assert(viewport.width > 1250 && viewport.height > 500, 'desktop simulation has a roomy viewport');
  await iframe.getByRole('button', { name: 'step', exact: true }).click();
  await page.waitForFunction(() => document.querySelector('#live-tick').textContent === '1');
  checks.push('real engine step reaches the parent telemetry');
  await iframe.getByRole('button', { name: 'run', exact: true }).click();
  await page.waitForFunction(() => Number(document.querySelector('#live-tick').textContent) > 1);
  await page.getByRole('tab', { name: 'Overview', exact: true }).click();
  await iframe.locator('button').filter({ hasText: /^run$/ }).waitFor({ state: 'attached' });
  await page.waitForFunction(() => ['Paused','Quiescent'].includes(document.querySelector('#live-status').textContent));
  checks.push('leaving live mode pauses the engine');
  await page.getByRole('tab', { name: 'Live simulation', exact: true }).click();
  await iframe.getByRole('button', { name: 'reset', exact: true }).click();
  await iframe.locator('.status.ready').waitFor();
  await page.waitForFunction(() => document.querySelector('#live-tick').textContent === '0');
  checks.push('reset reloads the original at tick zero');
  await page.screenshot({ path: path.join(output, 'live-desktop.png') });
  await page.locator('#toggle-properties').click();
  assert(await page.locator('#detail-properties').isVisible(), 'details can be restored alongside the live engine');
  await page.locator('#toggle-properties').click();
  await page.setViewportSize({ width: 390, height: 844 });
  assert(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), 'mobile workspace fits the screen');
  await iframe.getByRole('button', { name: 'step', exact: true }).click();
  await page.waitForFunction(() => document.querySelector('#live-tick').textContent === '1');
  await page.screenshot({ path: path.join(output, 'live-mobile.png') });
  await page.getByRole('tab', { name: 'Overview', exact: true }).click();
  await page.waitForFunction(() => { const image=document.querySelector('.overview-art img');return image?.complete && image.naturalWidth>0 && image.getBoundingClientRect().height>200; });
  checks.push('mobile preview retains a visible textured image');
  // The reported URLs must reach the app, including explicit unsupported-world errors.
  const userCases = [];
  for (const sha of ['3f5adf997012ab95a0293fcdd16e4de749547f3739ffce921c09f63aa8f0af0b','6130b77d5b51cb1cdfe0b86528c441d58a93d3c9db91178fe4e706ce2a1c92b1','9ba0f2e5d68c9cee4f0c20bb0674e2792463c225f63de8c4b42384df515db5fd']) {
    await page.goto(`${base}/sim-lab/?archive=${sha}&embed=atlas`);
    await page.locator('.status.ready,.status.error').waitFor({ timeout: 90000 });
    userCases.push({ sha, state: await page.locator('.status').getAttribute('class'), message: await page.locator('.status').innerText() });
  }
  assert(failures.length === 0, 'no missing local assets or API responses');
  assert(errors.length === 0, 'no uncaught browser errors');
  const result = { status: 'pass', checks, errors, failures, userCases, browser: browser.version(), scope: 'Asset serving, responsive workspace and real live-engine controls; not a new vanilla parity proof.' };
  await writeFile(path.join(output, 'result.json'), JSON.stringify(result,null,2)+'\n');console.log(JSON.stringify(result));
} catch(error) { await writeFile(path.join(output,'failure.json'),JSON.stringify({error:String(error),checks,errors,failures},null,2));throw error; }
finally { await browser.close(); }
