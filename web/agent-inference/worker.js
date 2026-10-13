import init, { Model } from './pkg/beverly_agent_inference.js';

const cacheName = 'beverly-tinyllama-v1-q4-verified';
let model;
let wasm;
let loading;
let active = false;
let cancelled = false;
let controller;

const send = (type, detail = {}) => self.postMessage({ type, ...detail });
const tick = () => new Promise(resolve => setTimeout(resolve, 0));

async function sha256(bytes) {
  const hash = await crypto.subtle.digest('SHA-256', bytes);
  return [...new Uint8Array(hash)].map(byte => byte.toString(16).padStart(2, '0')).join('');
}

async function asset(specification, cache, signal, offset = 0, total = specification.bytes) {
  const source = new URL(specification.url);
  if (source.origin !== 'https://huggingface.co' || !source.pathname.includes('/resolve/')) {
    throw new Error('Model assets must use the explicitly configured HTTPS Hugging Face hosting.');
  }
  let cached;
  try { cached = await cache?.match(specification.url); } catch {}
  if (cached) {
    send('status', { stage: 'Checking cached model assets' });
    const bytes = await cached.arrayBuffer();
    if (bytes.byteLength === specification.bytes && (!specification.sha256 || await sha256(bytes) === specification.sha256)) {
      send('progress', { stage: 'Model files present', fraction: (offset + bytes.byteLength) / total, bytes: offset + bytes.byteLength, total, cached: true });
      return bytes;
    }
    await cache.delete(specification.url);
    send('notice', { message: 'A cached asset was corrupt and will be downloaded again.' });
  }
  const response = await fetch(specification.url, { signal, cache: 'no-store', credentials: 'omit' });
  if (!response.ok) throw new Error(`Model download failed (HTTP ${response.status}).`);
  const reader = response.body?.getReader();
  if (!reader) throw new Error('This browser cannot stream model downloads.');
  const bytes = new Uint8Array(specification.bytes);
  let received = 0;
  let lastUpdate = 0;
  while (true) {
    const { done, value } = await reader.read();
    if (done) break;
    if (received + value.byteLength > specification.bytes) throw new Error('The downloaded model has an unexpected size.');
    bytes.set(value, received);
    received += value.byteLength;
    if (performance.now() - lastUpdate > 100) {
      send('progress', { stage: 'Downloading model assets', fraction: (offset + received) / total, bytes: offset + received, total, cached: false });
      lastUpdate = performance.now();
    }
    if (received === specification.bytes) {
      try { await reader.cancel(); } catch {}
      break;
    }
  }
  if (received !== specification.bytes) throw new Error('The download was interrupted. Please retry.');
  send('status', { stage: 'Verifying model integrity' });
  if (specification.sha256 && await sha256(bytes) !== specification.sha256) throw new Error('Model integrity verification failed. Please retry.');
  if (cache) {
    try { await cache.put(specification.url, new Response(bytes)); }
    catch { send('notice', { message: 'Browser storage is full or restricted. The model will work for this session but may need downloading next time.' }); }
  }
  send('progress', { stage: offset + received === total ? 'Download complete' : 'Downloading model assets', fraction: (offset + received) / total, bytes: offset + received, total, cached: false });
  return bytes.buffer;
}

async function load() {
  if (model) { send('ready'); return; }
  if (loading) return loading;
  controller = new AbortController();
  loading = (async () => {
    const started = performance.now();
    if (!self.WebAssembly || !self.crypto?.subtle) throw new Error('This browser does not support the required secure WebAssembly APIs.');
    if (navigator.deviceMemory && navigator.deviceMemory < 4) throw new Error('This device reports insufficient memory for TinyLlama. Try a desktop browser or the native macOS app.');
    wasm = await init();
    const manifestResponse = await fetch(new URL('./model.json', import.meta.url));
    if (!manifestResponse.ok) throw new Error('Model configuration is unavailable.');
    const manifest = await manifestResponse.json();
    let cache;
    try { cache = await caches.open(cacheName); }
    catch { send('notice', { message: 'Persistent storage is unavailable. Downloads will be kept only for this session.' }); }
    try {
      const estimate = await navigator.storage?.estimate();
      if (estimate?.quota && estimate.quota - estimate.usage < manifest.weights.bytes + manifest.tokenizer.bytes) {
        send('notice', { message: 'Available browser storage may be too small to cache this model. It can still run for this session.' });
      }
    } catch {}
    const downloadStarted = performance.now();
    const total = manifest.weights.bytes + manifest.tokenizer.bytes;
    const weights = await asset(manifest.weights, cache, controller.signal, 0, total);
    const tokenizer = await asset(manifest.tokenizer, cache, controller.signal, manifest.weights.bytes, total);
    const downloadMs = performance.now() - downloadStarted;
    send('status', { stage: 'Initializing model' });
    await tick();
    const initStarted = performance.now();
    model = new Model(new Uint8Array(weights), new Uint8Array(tokenizer));
    send('ready', { downloadMs, initializationMs: performance.now() - initStarted, loadMs: performance.now() - started, wasmMemoryBytes: wasm.memory.buffer.byteLength });
  })();
  try { await loading; }
  finally { loading = null; }
}

async function generate(prompt) {
  if (active) throw new Error('A response is already being generated.');
  if (!model) throw new Error('Load TinyLlama before sending a message.');
  active = true;
  cancelled = false;
  const started = performance.now();
  let firstTokenAt;
  let previous = '';
  let tokens = 0;
  try {
    const total = model.start(prompt);
    send('started', { promptTokens: total });
    while (!cancelled) {
      const result = JSON.parse(model.step());
      if (result.prefill) send('prefill', { completed: result.prefill, total: result.total });
      if (result.tokens) {
        if (firstTokenAt === undefined) { firstTokenAt = performance.now(); send('first-token', { elapsedMs: firstTokenAt - started }); }
        tokens = result.tokens;
      }
      if (result.text !== undefined && result.text !== previous) {
        send('token', { text: result.text, chunk: result.text.startsWith(previous) ? result.text.slice(previous.length) : '', tokens });
        previous = result.text;
      }
      if (result.done) break;
      await tick();
    }
    model.finish_turn();
    const elapsedMs = performance.now() - started;
    send('done', { cancelled, tokens, elapsedMs, ttftMs: firstTokenAt === undefined ? null : firstTokenAt - started, tokensPerSecond: firstTokenAt === undefined ? null : Math.max(0, tokens - 1) / Math.max(0.001, (performance.now() - firstTokenAt) / 1000), wasmMemoryBytes: wasm.memory.buffer.byteLength });
  } finally { active = false; }
}

self.addEventListener('message', async ({ data }) => {
  try {
    switch (data.command) {
      case 'load': await load(); break;
      case 'prompt': await generate(data.prompt); break;
      case 'cancel': cancelled = true; controller?.abort(); break;
      case 'reset': if (active) throw new Error('Stop generation before starting a new conversation.'); model?.reset(); send('reset'); break;
      default: throw new Error('Unknown worker command.');
    }
  } catch (error) {
    if (!model) controller?.abort();
    send('error', { fatal: error instanceof WebAssembly.RuntimeError, message: error.name === 'AbortError' ? 'Download cancelled. Retry when ready.' : String(error.message || error) });
  }
});