import assert from 'node:assert/strict';
import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);
const { chromium } = require(process.env.PLAYWRIGHT_MODULE || 'playwright');
const { PNG } = require(process.env.PNGJS_MODULE || 'pngjs');
const origin = process.env.AGENT_TEST_ORIGIN || 'http://127.0.0.1:3002';
const browser = await chromium.launch({ headless: true, args: process.platform === 'darwin' ? ['--enable-gpu', '--use-angle=metal'] : [] });

function headingTop(buffer) {
  const image = PNG.sync.read(buffer);
  const center = Math.floor(image.width / 2);
  for (let vertical = 72; vertical < image.height / 2; vertical++) {
    let ink = 0;
    for (let horizontal = center - 100; horizontal < center + 100; horizontal++) {
      const offset = (vertical * image.width + horizontal) * 4;
      if (image.data[offset] < 100 && image.data[offset + 1] < 100 && image.data[offset + 2] < 100) ink++;
    }
    if (ink >= 6) return vertical;
  }
  throw new Error('Could not locate the model-picker heading.');
}

try {
  for (const viewport of [{ width: 1280, height: 900 }, { width: 390, height: 844 }]) {
    const context = await browser.newContext({ viewport, deviceScaleFactor: 1 });
    await context.route('**/inference/worker.js', route => route.fulfill({
      contentType: 'application/javascript',
      body: 'self.loaded = false; self.addEventListener("message", ({data}) => { if (data.command === "load") { self.loaded = true; self.postMessage({type:"progress",stage:"Downloading model assets",fraction:0}); } });',
    }));
    const page = await context.newPage();
    const workerReady = page.waitForEvent('worker');
    await page.goto(`${origin}/assets/agent-demo/index.html`);
    await page.locator('#loading-screen').waitFor({ state: 'hidden', timeout: 120000 });
    const worker = await workerReady;
    const frames = count => page.evaluate(count => new Promise(resolve => {
      let elapsed = 0;
      function frame() { if (++elapsed >= count) resolve(); else requestAnimationFrame(frame); }
      requestAnimationFrame(frame);
    }), count);
    await frames(45);
    await page.mouse.move(viewport.width / 2, viewport.height / 2);
    await page.mouse.down();
    await frames(2);
    await page.mouse.up();
    await page.mouse.move(1, 1);
    await frames(45);
    assert.equal(await worker.evaluate(() => self.loaded), true, 'The loading-state test must select the model.');

    const positions = [];
    for (const fraction of [0.01, 0.09, 0.1, 0.11, 0.49, 0.5, 0.99, 1]) {
      await worker.evaluate(fraction => self.postMessage({ type: 'progress', stage: 'Downloading model assets', fraction }), fraction);
      await frames(20);
      positions.push({ fraction, top: headingTop(await page.screenshot()) });
    }
    for (let sample = 0; sample < 8; sample++) {
      await frames(20);
      positions.push({ fraction: 1, top: headingTop(await page.screenshot()) });
    }
    await worker.evaluate(() => self.postMessage({ type: 'progress', stage: 'Download complete', fraction: 1 }));
    await frames(30);
    positions.push({ phase: 'downloaded', top: headingTop(await page.screenshot()) });
    await worker.evaluate(() => self.postMessage({ type: 'status', stage: 'Initializing model' }));
    await frames(30);
    positions.push({ phase: 'initializing', top: headingTop(await page.screenshot()) });
    console.log(JSON.stringify({ viewport: viewport.width, positions }));
    assert.equal(new Set(positions.map(position => position.top)).size, 1, 'Download updates or hidden dots shifted the centered content.');
    await context.close();
  }
} finally {
  await browser.close();
}