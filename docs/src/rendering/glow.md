# Glow

`OuterGlow` adds a shader-rendered halo outside a `Surface`. Unlike a [drop shadow](shadows.md), it has no directional offset: the glow surrounds the surface's shape.

Use it for restrained emphasis, luminous accents, or an active surface. `BeverlyPlugin` installs the renderer required for these examples.

## Basic Usage

```rust
use beverly::rendering::{OuterGlow, Paint, ShadowFalloff, Surface};
use bevy::prelude::*;

fn add_glowing_surface(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        Node {
            width: Val::Px(300.0),
            height: Val::Px(110.0),
            ..default()
        },
        Surface::rounded_rect_fill(8.0, Paint::solid(Color::WHITE))
            .outer_glow(
                OuterGlow::new(Color::srgb(0.10, 0.78, 0.48))
                    .with_blur(24.0)
                    .with_spread(4.0)
                    .with_opacity(0.65)
                    .with_falloff(ShadowFalloff::Gaussian),
            ),
    ));
}
```

The glow follows the surface's rounded shape. It does not change its fill, resize the node, or reserve spacing around neighboring content.

## Parameters

| Parameter | Effect |
| --- | --- |
| `color` | Halo tint |
| `blur` | Softness in logical pixels |
| `spread` | Expands the halo silhouette when positive; contracts it when negative |
| `opacity` | Effect strength from 0.0 to 1.0 |
| `falloff` | Fade profile outside the silhouette |

`OuterGlow::new(color)` defaults to a 10-pixel blur, zero spread, Gaussian falloff, and **zero opacity**. Set `.with_opacity(...)` explicitly to make it visible.

Increasing blur softens the halo; increasing spread makes it extend farther before fading. For a tightly defined luminous edge, try a small blur and spread. For a softer atmosphere, use a larger blur with restrained opacity.

## Falloff Modes

The glow shares `ShadowFalloff` with the shadow effects:

- `Linear`: a linear fade.
- `Smooth`: a smooth fade.
- `Gaussian`: a Gaussian-style fade, used by the default glow.

Falloff changes the fade profile independently of blur and spread. These are shader effects, not CSS filter declarations; inspect the rendered result when tuning them.

## Runtime Updates

Change `surface.effects.outer_glow` to update the effect:

```rust
use beverly::rendering::{OuterGlow, Surface};
use bevy::prelude::*;

fn set_glow_strength(surface: &mut Surface, color: Color, opacity: f32) {
    surface.effects.outer_glow = Some(
        OuterGlow::new(color)
            .with_blur(24.0)
            .with_spread(4.0)
            .with_opacity(opacity.clamp(0.0, 1.0)),
    );
}
```

Use `surface.clear_outer_glow()` to remove it. Each surface has one outer-glow slot; setting another value replaces the previous glow. Borders, outer shadows, and inner shadows are separate treatments and can coexist with it.

The effect is static unless an application system changes its parameters. If animating or pulsing it, honor reduced-motion preferences.

## Layout and Theme

Leave room around a glowing surface: the effect does not reserve layout space, and a clipping ancestor can cut off the halo. Nearby glows can overlap.

Explicit glow colors remain unchanged when light/dark mode switches. Read `ThemeResource` when the tint should follow theme tokens. A halo usually appears stronger against a dark background, so compare both themes before choosing its opacity.

Keep text contrast sufficient against the surface fill. Glow should supplement, not replace, a visible focus ring or other semantic state indicator.

## Demo

Run the interactive glow demo from the repository root:

```sh
cargo run --example glow
cargo run --example glow -- dark
```

The demo includes Mint, Coral, Sky, and Gold color swatches; softness, spread, and opacity sliders; and Linear, Smooth, and Gaussian falloff selectors.

The main preview follows all controls. The three reference samples use the selected color with fixed 16-pixel blur, 2-pixel spread, and Gaussian falloff:

| Sample | Opacity |
| --- | --- |
| Subtle | 0.2 |
| Bright | 0.5 |
| Radiant | 0.9 |

These sample names are demo comparisons, not built-in `OuterGlow` presets. The top-right button switches light/dark mode; surface fills follow the theme while the selected glow color remains unchanged.