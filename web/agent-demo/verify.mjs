import assert from 'node:assert/strict';
import fs from 'node:fs';
import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);
const { chromium } = require(process.env.PLAYWRIGHT_MODULE || 'playwright');
const origin = process.env.AGENT_TEST_ORIGIN || 'http://127.0.0.1:3002';
const manifest = JSON.parse(fs.readFileSync(new URL('../agent-inference/model.json', import.meta.url)));
const browser = await chromium.launch({ headless: true, args: process.platform === 'darwin' ? ['--enable-gpu', '--use-angle=metal'] : [] });

try {
  for (const viewport of [{ width: 1280, height: 900 }, { width: 390, height: 844 }]) {
    const context = await browser.newContext({ viewport, deviceScaleFactor: 2 });
    const page = await context.newPage();
    const errors = [];
    const modelRequests = [];
    context.on('request', request => { if (request.url().includes('huggingface.co')) modelRequests.push(request.url()); });
    page.on('pageerror', error => errors.push(String(error)));
    await page.goto(`${origin}/index.html#download`);
    await page.locator('[data-browser-demo-trigger]').scrollIntoViewIfNeeded();
    const before = await page.evaluate(() => ({ scroll: scrollY, source: document.querySelector('#browser-demo-frame').getAttribute('src') }));
    assert.equal(before.source, null);
    await page.locator('[data-browser-demo-trigger]').click();
    await page.frameLocator('#browser-demo-frame').locator('#loading-screen').waitFor({ state: 'hidden', timeout: 120000 });
    assert.equal(modelRequests.length, 0, 'Model download must wait for explicit selection.');
    assert.match(await page.locator('#browser-demo-notice .browser-demo-modal-title').textContent(), /Easy there, speed racer\./);
    assert.match(await page.locator('#browser-demo-notice .browser-demo-modal-text').textContent(), /TinyLlama locally in your browser using WASM/);
    assert.match(await page.locator('#browser-demo-notice .browser-demo-modal-text').textContent(), /trying its little heart out\. 🐢/);
    assert.match(await page.locator('#browser-demo-dismiss-notice').textContent(), /I’m In No Rush/);
    assert.match(await page.locator('#browser-demo-modal-close-popup').textContent(), /try the desktop version/);
    assert.doesNotMatch(await page.locator('#browser-demo-notice').textContent(), /\[close popup\]/);
    const textAligns = await page.evaluate(() => ({
      title: getComputedStyle(document.querySelector('.browser-demo-modal-title')).textAlign,
      text: getComputedStyle(document.querySelector('.browser-demo-modal-text')).textAlign,
    }));
    assert.equal(textAligns.title, 'center');
    assert.equal(textAligns.text, 'left');
    assert.equal(await page.locator('#browser-demo-notice .browser-demo-modal-icon svg').isVisible(), true);
    await page.locator('#browser-demo-dismiss-notice').click();
    assert.equal(await page.locator('#browser-demo-notice').isVisible(), false);
    await page.frameLocator('#browser-demo-frame').locator('#bevy').focus();
    await page.keyboard.press('Escape');
    await page.waitForFunction(() => !document.querySelector('#browser-demo-dialog').open && document.body.style.position !== 'fixed');
    const after = await page.evaluate(() => ({ scroll: scrollY, locked: document.body.style.position, focus: document.activeElement.hasAttribute('data-browser-demo-trigger'), source: document.querySelector('#browser-demo-frame').getAttribute('src') }));
    assert.equal(after.scroll, before.scroll);
    assert.equal(after.locked, '');
    assert.equal(after.focus, true);
    assert.equal(after.source, '/assets/agent-demo/index.html');
    await page.locator('[data-browser-demo-trigger]').click();
    assert.equal(await page.locator('#browser-demo-notice').isVisible(), false);
    assert.deepEqual(errors, []);
    await context.close();
    console.log(`Overlay passed at ${viewport.width}px.`);
  }

  {
    const context = await browser.newContext();
    const page = await context.newPage();
    await page.goto(`${origin}/index.html#download`);
    await page.locator('[data-browser-demo-trigger]').scrollIntoViewIfNeeded();
    await page.locator('[data-browser-demo-trigger]').click();
    assert.equal(await page.locator('#browser-demo-notice').isVisible(), true);
    await page.locator('#browser-demo-modal-close-popup').click();
    await page.waitForFunction(() => !document.querySelector('#browser-demo-dialog').open && document.body.style.position !== 'fixed');
    assert.equal(await page.evaluate(() => document.querySelector('#browser-demo-dialog').open), false);
    await context.close();
    console.log('Close popup button verified.');
  }

  const context = await browser.newContext();
  const page = await context.newPage();
  const weightsUrl = manifest.weights.url;
  let downloads = 0;
  await context.route('https://huggingface.co/**', async route => {
    downloads++;
    await route.fulfill({ status: 503, body: 'Synthetic test network failure', headers: { 'access-control-allow-origin': '*' } });
  });
  await page.goto(`${origin}/web/agent-inference/harness.html`);
  await page.evaluate(async weightsUrl => {
    const cache = await caches.open('beverly-tinyllama-v1-q4-verified');
    await cache.put(weightsUrl, new Response('corrupt test asset'));
  }, weightsUrl);
  await page.locator('#load').click();
  await page.waitForFunction(() => events.some(event => event.type === 'error'), null, { timeout: 30000 });
  const failure = await page.evaluate(() => events.find(event => event.type === 'error'));
  assert.match(failure.message, /503/);
  assert(downloads > 0);
  assert.equal(await page.evaluate(async weightsUrl => Boolean(await (await caches.open('beverly-tinyllama-v1-q4-verified')).match(weightsUrl)), weightsUrl), false);
  await page.evaluate(() => { events = []; });
  await page.locator('#load').click();
  await page.waitForFunction(() => events.some(event => event.type === 'error'), null, { timeout: 30000 });
  assert(downloads > 1, 'Retry must attempt a new download.');
  await context.close();
  console.log('Corrupt-cache eviction, visible download failure, and retry passed.');
} finally {
  await browser.close();
}