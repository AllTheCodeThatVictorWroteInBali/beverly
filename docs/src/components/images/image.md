# Image

`image()` creates a UI image. Set its source with `.src(...)` and provide
accessible alternative text with `.alt(...)`:

```rust
use beverly::prelude::*;

let report = image()
    .src("photos/report.webp")
    .alt("Quarterly revenue chart")
    .width(640)
    .height(360);

let remote = image()
    .src("https://cdn.example.com/team/alex.png?size=256")
    .alt("Alex Chen");
```

The same API accepts a `Handle<Image>` for generated or in-memory assets, local
asset paths, Bevy `embedded://` paths, and external HTTP(S) URLs. An image that
conveys no information should use `.decorative(true)` instead of an empty
alternative description.

## Local SVG Assets

The package includes small SVG samples at `assets/examples/analytics.svg` and
`assets/examples/mark.svg`. In a Bevy application whose asset root is the
workspace `assets` directory, refer to these as relative paths:

```rust
image()
    .src("examples/analytics.svg")
    .alt("Line chart showing monthly usage increasing through the year")
    .width(640)
    .height(360)

image()
    .src("examples/mark.svg")
    .alt("Geometric Beverly mark")
    .width(96)
    .height(96)
```

SVG is rasterized at its intrinsic size. External SVG resources are disabled;
inline vector shapes and paths work, but linked images, external styles, and
external fonts are not fetched.

## Supported Formats

The built-in decoder supports PNG, JPEG, GIF, WebP, SVG, BMP, and ICO. These
formats are enabled by Beverly's default `image_formats` feature plus SVG
rasterization. AVIF is unsupported by the configured Bevy decoder and produces
an explicit load error. Animated GIF/WebP sources display as a decoded still
image.

Encoded downloads are limited to 16 MiB and raster output to 16 megapixels.
SVG output is subject to the same pixel limit. These limits reduce accidental
large allocations; they are not a complete defense against untrusted assets.

External images use Bevy's HTTP(S) asset readers. Browser servers must permit
the app's origin with CORS, and HTTPS apps require HTTPS image URLs to avoid
mixed-content blocking. Query parameters are preserved, fragments are removed,
and embedded credentials or non-HTTP(S) URL schemes are rejected. Apply
application-specific URL allowlists when sources are not trusted.

## Loading State

`ImageStatus` on the spawned image entity reports `Empty`, `Loading`, `Ready`,
or `Failed(message)`. Observe it to show a placeholder, retain a fallback, or
hide a decorative image while loading. Changing `.src(...)` loads the new
source and replaces the existing image. `Photo::new(Handle<Image>)` and avatar
image paths remain available for their specialized layouts.

## Accessibility

Provide meaningful alternative text for informative images:

```rust
image()
    .src("products/compass.webp")
    .alt("Brass pocket compass with its needle pointing north")
```

For decorative imagery, explicitly mark it:

```rust
image()
    .src("textures/paper-grain.svg")
    .decorative(true)
```

A missing or whitespace-only `alt` on a non-decorative image is reported as a
warning when spawned. Alternative text should describe the information the
image contributes, not repeat nearby captions.

## Live Demo

<iframe src="../../assets/demos/index.html?demo=photo" title="Interactive Beverly photo demo" loading="lazy" allow="fullscreen" style="display: block; width: 100%; height: min(820px, 90vh); min-height: 580px; border: 0;"></iframe>

<a href="../../assets/demos/index.html?demo=photo" target="_blank" rel="noopener">Open the demo in a separate tab</a>.
