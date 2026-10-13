# Beverly Agent Browser Demo

The native monitor UI and controller are reused from
`apps/beverly-agent/src/bin/monitor.rs`. Native builds retain Candle 0.11,
Metal, HF filesystem caching, and the OS worker thread. The browser UI uses
`apps/beverly-agent/src/web_platform.rs` to adapt its typed commands/events to
a dedicated module Worker. It never calls an inference API.

## Model and Runtime

The native model is `TinyLlama/TinyLlama-1.1B-Chat-v1.0`, F16 safetensors
(2,200,119,864 bytes). That single allocation exceeds wasm32's Rust allocation
limit, and F32 weights would exceed Wasm linear memory. The browser therefore
uses Candle 0.11's existing `quantized_llama::ModelWeights` CPU implementation
with the Q4_0 GGUF conversion published by TheBloke for the same checkpoint.
This is lossy quantization, not bit-identical native weights; answers can differ.
The original tokenizer is retained. Exact revisions, sizes, URLs and SHA-256
hashes are pinned in `web/agent-inference/model.json`.

Weights: **637,699,456 bytes**. Tokenizer: **1,842,767 bytes**.
Both are integrity-checked, fetched with no credentials, and cached with Cache
Storage. Missing/corrupt entries are downloaded again. Storage failure is
reported and the model remains usable in memory for that session. No model
bytes are included in the app bundle or downloaded on the initial website load.

The chat template, system prompt, EOS token, BOS behavior, seed `299792458`,
temperature `0.7`, top-p `0.9`, and 256-token generation limit match native
TinyLlama. Shared Rust prompt construction adds alternating user/assistant
history. Context is capped at 1792 prompt tokens plus 256 output tokens;
users can start a new chat when the context is full.

Candle 0.11's Wasm SIMD build currently fails with missing F16/BF16 CPU helper
types. This implementation uses its working scalar CPU path. No WebGPU,
SharedArrayBuffer, additional inference engine, or multithreaded Wasm is used.
Token generation and 32-token prefill chunks run only in the worker. Cancel is
processed between steps; a long synchronous prefill step delays cancellation.
The measured cancel-during-prefill test completed after 43.47 s with zero
generated tokens. Cancellation is not a hard real-time interrupt.
Closing/hiding the overlay requests cancellation but keeps the conversation
and model instance, so reopening does not lose the session.

### Reproducing the Conversion

The tested deployed artifact is the pinned, SHA-256-verified TheBloke conversion,
whose model card identifies the TinyLlama v1.0 source checkpoint. To create an
independent conversion using llama.cpp:

```sh
hf download TinyLlama/TinyLlama-1.1B-Chat-v1.0 \
  --revision fe8a4ea1ffedaf415f4da2f062534de366a451e6 --local-dir tinyllama-source
python llama.cpp/convert_hf_to_gguf.py tinyllama-source \
  --outfile tinyllama-f16.gguf --outtype f16
llama.cpp/build/bin/llama-quantize tinyllama-f16.gguf tinyllama-q4_0.gguf Q4_0
```

Pin the converter revision, compute the new size/hash, update the manifest, and
rerun browser tests before deploying a newly generated artifact. These commands
describe the reproducible format conversion; the independently regenerated
artifact was not used for the measurements below.

## Build and Host

Prerequisites: Rust `wasm32-unknown-unknown`, Trunk 0.21.14, and wasm-bindgen CLI
0.2.129 (the build script can install the matching CLI). From the repo root:

```sh
bash web/agent-demo/build.sh
python3 -m http.server 3002 --bind 127.0.0.1
```

Visit `http://127.0.0.1:3002/index.html#download`. Production uses
`/assets/agent-demo/index.html`; its inference worker lives under `inference/`.
The Pages workflow packages the full bundle and caches it independently of
the existing component demos. Large model weights stay on the explicitly
configured, revision-pinned HTTPS Hugging Face hosting, not GitHub Pages.

The standalone inference proof is at
`/web/agent-inference/harness.html`. Run `bash web/agent-inference/build.sh`
before opening it.

The browser smoke test uses Playwright (install it in a temporary tools prefix
or an existing test environment, then pass its module path):

```sh
PLAYWRIGHT_MODULE=/tmp/beverly-demo-qa/node_modules/playwright \
  node web/agent-demo/verify.mjs
```

It checks desktop/mobile overlay state, lazy downloads, notice dismissal,
Escape/scroll/focus restoration, corrupt-cache eviction, and network retry.
The network-error cases are explicitly simulated; they are not inference
benchmarks. The independent harness performs the real inference tests.

The loading-layout regression test measures the heading's screenshot pixel
position across percentage updates and hidden-dot animation. It uses a mocked
worker and does not download a model:

```sh
PLAYWRIGHT_MODULE=/tmp/beverly-demo-qa/node_modules/playwright \
PNGJS_MODULE=/tmp/beverly-demo-qa/node_modules/pngjs \
  node web/agent-demo/verify-loading.mjs
```

The send/pause visual test checks the transparent control, visible muted icon,
and clickable pause action in a mocked chat at desktop/mobile sizes:

```sh
PLAYWRIGHT_MODULE=/tmp/beverly-demo-qa/node_modules/playwright \
PNGJS_MODULE=/tmp/beverly-demo-qa/node_modules/pngjs \
  node web/agent-demo/verify-submit-style.mjs
```

HTTPS (or localhost for development) is required for Cache Storage and
Web Crypto. **COOP/COEP headers and cross-origin isolation are not required.**
The app document provides a scoped CSP allowing same-origin workers/Wasm and
the configured Hugging Face asset CDN domains. It does not change the site's
global policy or the existing demos. Browser Wasm memory errors are reported;
there is no cloud fallback. Prompts are not logged, sent to a provider, or used
for analytics. Quantized assets are public and no API key is needed.

On coarse-pointer browsers a transparent native textarea is positioned over
the Beverly composer and mirrored into its existing Rust component. This is a
platform keyboard/IME/accessibility bridge, not a replacement chat renderer.
It supports Unicode, composition, selection and Enter submission. The website
uses `visualViewport` for on-screen keyboard layout changes. Real mobile
hardware keyboard behavior still needs device testing.

Generated module-preload SRI remains enabled. `refresh-assets.py` recomputes
snippet digests when static packaging refreshes the JavaScript bridge; it does
not weaken the CSP or remove integrity checking.

## Measured Proof Results

Measured on the available Apple M4 development Mac with headless Chromium,
single-threaded scalar Candle, not a real mobile device:

| Measurement | Observed |
|---|---:|
| Cold download + integrity checks | 132.63 s |
| Cold model initialization | 2.12 s |
| Total cold load, including runtime startup | 134.80 s |
| Warm load, including cache hash validation | 3.05 s |
| Allocated inference Wasm memory | 1,589,313,536 bytes |
| First Paris-answer token | 32.37 s |
| Seven-token Paris answer, total generation | 38.54 s |
| Sustained rate after first token | 0.97 tokens/s |
| Packaged UI + worker Wasm (uncompressed) | about 33 MiB + 4 MiB |

A measured complete static package, including glue/scripts/logo/metadata,
contained 34,476,082 UI bytes and 4,243,844 worker bytes. The worker Wasm is
loaded only when TinyLlama is selected. There are no model weight files in
the static app. Minor UI rebuilds can change these exact byte counts.

Both desktop and web TTFT labels use milliseconds through 1000 ms, then
seconds with two decimal places for longer measurements.

The packaged overlay's end-to-end Paris test measured 35.66 s TTFT and
0.91 tokens/s. Two-turn name recall returned "Ada" on the second turn;
TTFTs were 61.16 s and 84.27 s, rates 0.56 and 0.87 tokens/s. A 50 ms UI
heartbeat ran 3440 times during those generations, with **zero network
requests during inference**. Timings depend strongly on prompt length, browser,
CPU load and device. Wasm memory is allocated linear memory, not total process
RSS, which includes UI buffers, cached/downloaded assets and browser overhead.

Initial interrupted download produced a visible error; retry succeeded.
Cache reuse, native prompt tests, overlay lazy loading, restored scroll/focus,
Escape from the canvas, and retained state on reopening were tested.
The final phone-emulated native editor sent each prompt exactly once and
generated "The capital of France is Paris." followed by "Paris." for the
contextual second turn. TTFTs were 41.64 s and 77.36 s, with rates 0.76 and
0.64 tokens/s. An explicitly simulated fatal worker event returned the UI to
its retry screen. It is a failure-handling test, not a claim that a real OOM
was reproduced.

Real mobile device compatibility is **unverified**. Phone viewport layout is
tested by emulation only. Small-memory devices are rejected when the browser
reports less than 4 GiB; browsers that do not report memory must attempt loading
and may fail. Background suspension can delay status updates/cancellation.
Safari and GPU acceleration are not claimed. Playwright WebKit on the same Mac
successfully generated the Paris answer: 149.27 s total cold load, 6.69 s model
initialization, 96.85 s TTFT, and 0.37 tokens/s. Its initial large-Blob download
failed; a single pre-sized download buffer fixed that failure and reduced peak
download memory. WebKit warm loading took 18.60 s, including 13.92 s cached
asset reading/hash validation and 4.33 s initialization, with both assets
reused and zero model download requests. Firefox's test binary could not launch due to its profile
folder error, so Firefox compatibility remains unverified.

The native monitor built with its locked dependencies and launched without an
inference/runtime error. Production publication and real-device mobile testing
remain release gates; local tests use the exact production `/assets/agent-demo/`
paths rather than a localhost inference endpoint.