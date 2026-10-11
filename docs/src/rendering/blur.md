# Blur

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

Blur is one of the most effective ways to show depth and separation in a layered interface. Used sparingly, it helps create readable glass or ambient surfaces without reducing legibility. In a product UI, blur is most useful when it reinforces hierarchy and layering rather than when it becomes a decorative effect on its own.

## What blur does in UI

A blurred surface suggests distance, depth, or separation from the content beneath it. It works well for:

- modal backdrops
- floating utility surfaces
- translucent overlays and chrome
- layered panels that need subtle separation
- premium product surfaces that want a glass-like feel

The key is that blur should support the interface’s readability, not undermine it. If text becomes hard to read, the surface has crossed from excellent layering into bad legibility.

## Backdrop blur

A `Backdrop` blurs and adjusts the pixels already rendered **behind** a surface, rather than blurring the surface's own fill. Attach it with `Surface::with_backdrop`; `BeverlyPlugin` installs the renderer and capture pass.

```rust
use beverly::rendering::{Backdrop, BackdropQuality, Paint, Surface};
use bevy::prelude::*;

fn add_frosted_panel(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        Node {
            width: Val::Px(310.0),
            height: Val::Px(180.0),
            padding: UiRect::all(Val::Px(22.0)),
            ..default()
        },
        Surface::rounded_rect_fill(8.0, Paint::solid(Color::NONE))
            .uniform_border(1.0, Paint::solid(Color::WHITE.with_alpha(0.65)))
            .with_backdrop(
                Backdrop::new()
                    .with_blur(18.0)
                    .with_tint(Color::srgb(0.96, 0.96, 0.96))
                    .with_tint_opacity(0.20)
                    .with_quality(BackdropQuality::High),
            ),
    ));
}
```

The blur follows the surface's shape, including rounded corners. The example uses a transparent fill so the frosted backdrop is the visible material. A fill, border, or shadow on the same surface draws over it.

### Parameters

| Setter | Effect |
| --- | --- |
| `with_blur(pixels)` | Blur radius in pixels, capped at 128. `0` disables blurring |
| `with_tint(color)` | Color mixed over the blurred backdrop |
| `with_tint_opacity(value)` | Tint strength from 0.0 to 1.0; also scaled by the tint color's alpha |
| `with_saturation(factor)` | Color intensity; `1.0` is unchanged, `0.0` is greyscale |
| `with_brightness(factor)` | Brightness; `1.0` is unchanged |
| `with_contrast(factor)` | Contrast; `1.0` is unchanged |
| `with_quality(quality)` | Blur sample count: `Low`, `Medium` (default), or `High` |

Saturation, brightness, and contrast are clamped to 0.0–2.0. A default `Backdrop::new()` has no visible effect until at least one parameter changes. Use `.with_liquid_glass(...)` to add refraction; see [Liquid Glass](liquid-glass.md).

### Quality

Quality controls how many samples the blur takes over its radius, in addition to the center sample:

- `Low`: 8 samples
- `Medium`: 16 samples
- `High`: 32 samples

Higher quality produces smoother frosting, especially with a large radius or high-contrast content behind the surface. It also costs more GPU time, so reserve `High` for a few prominent surfaces.

### What gets blurred

Backdrop surfaces share one snapshot of the UI taken before the first surface that samples it is drawn. This has a few consequences:

- Only content drawn **before** the blurred surface appears in the blur.
- A backdrop surface does not blur another backdrop surface; stacked glass does not accumulate.
- SVG icons and later overlay cameras are not captured.
- Beverly supports a single full-window camera for capture.

Place the content that should be blurred behind the surface in the same UI tree, earlier in draw order.

### Updating at runtime

Replace `Surface::backdrop` to change the effect:

```rust
use beverly::rendering::{Backdrop, Surface};
use bevy::prelude::*;

fn set_blur(surface: &mut Surface, blur: f32) {
    surface.backdrop = Some(Backdrop::new().with_blur(blur).with_tint_opacity(0.2));
}
```

Use `surface.clear_backdrop()` to remove it. An explicit tint color is not replaced when the theme changes; build it from `ThemeResource` when it should follow light or dark mode.

### Developer diagnostics

Two environment variables help while debugging:

- `UI_BACKDROP_DISABLED=1` turns off backdrop capture and sampling.
- `UI_BACKDROP_DEBUG=source`, `blur`, or `mask` shows the captured backdrop, the blurred result, or the surface mask.

## Demo

Run the interactive backdrop-blur demo from the repository root:

```sh
cargo run --example backdrop_blur
cargo run --example backdrop_blur -- dark
```

The demo places two windows over a moving, striped background: a clear view with a light dark tint, and a frosted view. Sliders adjust the frosted window's blur (0–64 px), tint (0–100%), and saturation (0–200%). The frosted view uses `High` quality, and the stripes hold still when reduced motion is enabled.

Compare the two windows to see how blur softens the edges behind the frosted one. The top-right toggle switches light and dark mode, which changes the frosted tint color.

## Use cases

- modal overlays
- floating panes and sidebars
- translucent chrome and utility surfaces
- layered data views with a quiet background
- premium application surfaces that need a softened, atmospheric layer

## Accessibility note

If blur reduces contrast too much, the surface should fall back to a more opaque treatment or a higher-contrast border. In accessibility-driven design, blur cannot be allowed to hide critical text or destroy state readability.

## Blur and context

The best use of blur is usually conservative. A little blur can create a sense of separation and focus, but too much blur will make a layer feel vague and reduce confidence in the interface. In dense workspaces, the user needs to understand the structure at a glance.

This means blur should be used with clear boundaries:

- behind modal shells or drawers
- around focus regions or transient surfaces
- on layered chrome that should feel ambient rather than dominant
- not on every text-heavy or dense data surface by default

## Performance and visual cost

Blur is often more expensive than a flat fill or a simple shadow because it requires more GPU work and more blending. That does not make it invalid; it just makes proper scope and restraint important.

A thoughtful bloom or blur policy keeps the effect active only where it adds value. This is especially important for dashboards, data-heavy screens, and mobile or lower-power devices.

## The design principle

Blur is most successful when it communicates depth without hiding structure. It should make the interface feel layered and refined, while still allowing the user to read, focus, and act with confidence.

Used correctly, blur is a quiet luxury. Used without constraint, it becomes a readability problem.
