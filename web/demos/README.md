# Documentation Demos

The shared Wasm binary runs the real Rust examples, selected with
`index.html?demo=checkbox` (the value is the example's filename without `.rs`).
All demos share compiled dependencies and browser downloads. The existing
button showcase keeps its dedicated bundle.

## Build

Install the `wasm32-unknown-unknown` target, Trunk `0.21.14`, and mdBook:

```sh
mdbook build docs
bash web/demos/build.sh
SKIP_TRUNK_BUILD=true bash web/button-demo/build.sh
```

Serve `docs/book` over HTTP to view the embedded demos. For deployment beneath
`/docs`, use `TRUNK_PUBLIC_URL=/docs/assets/demos/` when building. The Pages
workflow handles both bundles, artifact caches, and compilation caches.
Use `bash web/button-demo/build.sh` without `SKIP_TRUNK_BUILD` when the button
bundle has not been built yet.

## Add a Demo

Add an entry in [manifest.json](manifest.json) pointing to the example and its
canonical documentation page, then add the iframe on that page. The build
checks that every example has an entry; `button` is the sole exception because
it already has its own bundle. The dispatcher is generated from the manifest.

Examples use `examples/support/mod.rs` and `DemoAppExt::run_demo()` to configure
the embedded canvas, loading state, responsive page framing, and touch scrolling
on the web. The browser implementation lives in `src/browser.rs` and is compiled
once for the whole bundle. Native examples still use the normal Bevy event loop.

Browser builds do not enable the native file-dialog integration. The file-input
demo displays an unavailable state rather than pretending to open a picker.