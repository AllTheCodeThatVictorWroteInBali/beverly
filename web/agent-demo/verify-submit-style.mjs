import assert from 'node:assert/strict';
import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);
const { chromium } = require(process.env.PLAYWRIGHT_MODULE || 'playwright');
const { PNG } = require(process.env.PNGJS_MODULE || 'pngjs');
const origin = process.env.AGENT_TEST_ORIGIN || 'http://127.0.0.1:3002';
const browser = await chromium.launch({ headless: true, args: process.platform === 'darwin' ? ['--enable-gpu', '--use-angle=metal'] : [] });

function sampleControl(buffer, point) {
  const image = PNG.sync.read(buffer);
  let darkPixels = 0;
  let iconPixels = 0;
  for (let vertical = point.y - 13; vertical <= point.y + 13; vertical++) {
    for (let horizontal = point.x - 13; horizontal <= point.x + 13; horizontal++) {
      const offset = (vertical * image.width + horizontal) * 4;
      const [red, green, blue] = image.data.subarray(offset, offset + 3);
      if (red < 70 && green < 70 && blue < 70) darkPixels++;
      if (red > 80 && red < 220 && blue >= red && blue < 235) iconPixels++;
    }
  }
  return { darkPixels, iconPixels };
}

try {
  for (const viewport of [{ width: 1280, height: 900 }, { width: 390, height: 844 }]) {
    const context = await browser.newContext({ viewport, deviceScaleFactor: 1 });
    await context.route('**/inference/worker.js', route => route.fulfill({
      contentType: 'application/javascript',
      body: 'self.loaded=false;self.cancelled=false;addEventListener("message",({data})=>{if(data.command==="load"){self.loaded=true;postMessage({type:"ready"});}if(data.command==="prompt")postMessage({type:"started",promptTokens:1});if(data.command==="cancel"){self.cancelled=true;postMessage({type:"done",cancelled:true});}});',
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
    const click = async (horizontal, vertical) => {
      await page.mouse.move(horizontal, vertical);
      await page.mouse.down();
      await frames(2);
      await page.mouse.up();
      await page.mouse.move(1, 1);
      await frames(30);
    };
    await frames(45);
    await click(viewport.width / 2, viewport.height / 2);
    assert.equal(await worker.evaluate(() => self.loaded), true);
    await frames(45);
    const panelWidth = Math.min(viewport.width, 760);
    const point = { x: Math.round((viewport.width + panelWidth) / 2 - 51), y: viewport.height - 113 };
    const sendImage = await page.screenshot({ path: `/tmp/beverly-send-transparent-${viewport.width}.png` });
    const send = sampleControl(sendImage, point);
    assert.equal(send.darkPixels, 0, 'Send control must not draw a black background.');
    assert(send.iconPixels > 2, 'Send icon must remain visible.');

    await click(viewport.width / 2, viewport.height - 145);
    await page.keyboard.type('Controlled style test');
    await page.keyboard.press('Enter');
    await frames(45);
    const pauseImage = await page.screenshot({ path: `/tmp/beverly-pause-transparent-${viewport.width}.png` });
    const pause = sampleControl(pauseImage, point);
    assert.equal(pause.darkPixels, 0, 'Pause control must not draw a black background.');
    assert(pause.iconPixels > 2, 'Pause icon must remain visible.');
    await click(point.x, point.y);
    assert.equal(await worker.evaluate(() => self.cancelled), true, 'Transparent pause control must remain clickable.');
    console.log(JSON.stringify({ viewport: viewport.width, send, pause, cancelClicked: true }));
    await context.close();
  }
} finally {
  await browser.close();
}