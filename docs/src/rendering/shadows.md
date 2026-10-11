# Shadows

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

Shadows help show elevation and surface separation. They are best when subtle and consistent, and they should support the interface without becoming decoration. In a well-designed system, shadows are not a visual gimmick; they are a way to communicate depth, layering, and separation in a controlled, readable way.

## What shadows communicate

A shadow is one of the clearest ways to tell a user that one surface sits above another. It helps the eye understand:

- which panel is floating
- which control is active or elevated
- which content is layered above the background
- which overlay needs stronger separation from the page beneath it

This makes shadows a useful tool for hierarchy, especially in dense or highly structured interfaces.

## Drop shadow sizes

Beverly keeps a small set of shadow presets so surfaces can communicate depth in a predictable way.

### Small

Use a small shadow for light lift: compact cards, hover states, quick actions, and other subtle surfaces that should feel slightly separated from the background.

### Regular

Use the regular shadow for the default elevated surface. This is the standard choice for cards, property panels, and other content that needs a clear separation but should still feel understated.

### Large

Use the large shadow for major floating overlays: modals, drawers, menus, and panels that need a strong sense of separation from the page beneath them.

## Example

```rust
use beverly::rendering::{OuterShadow, Paint, Surface};
use bevy::prelude::*;

fn elevated_surface(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        Node {
            width: Val::Px(240.0),
            height: Val::Px(120.0),
            ..default()
        },
        Surface::rounded_rect_fill(8.0, Paint::solid(Color::WHITE))
            .outer_shadow(OuterShadow::regular(Color::BLACK)),
    ));
}
```

`BeverlyPlugin` installs the shader-backed surface renderer. `Surface::outer_shadow` adds a drop shadow outside the surface's shape without changing its layout dimensions.

## Preset Values

Each preset accepts a shadow color and uses smooth falloff with zero spread:

| Constructor | Offset X / Y | Blur | Opacity |
| --- | --- | --- | --- |
| `OuterShadow::small(color)` | 0 / 2 px | 4 px | 0.075 |
| `OuterShadow::regular(color)` | 0 / 8 px | 16 px | 0.15 |
| `OuterShadow::large(color)` | 0 / 16 px | 48 px | 0.175 |

These are Bootstrap-style presets. They are a convenient starting point, not a requirement to use the same shadow on every surface.

## Custom Drop Shadow

```rust
use beverly::rendering::{OuterShadow, Paint, ShadowFalloff, Surface};
use bevy::prelude::*;

let shadow = OuterShadow::new(Color::BLACK)
    .with_offset(Vec2::new(0.0, 12.0))
    .with_blur(24.0)
    .with_spread(0.0)
    .with_opacity(0.35)
    .with_falloff(ShadowFalloff::Smooth);

let surface = Surface::rounded_rect_fill(8.0, Paint::solid(Color::WHITE))
    .outer_shadow(shadow);
```

| Parameter | Effect |
| --- | --- |
| `color` | Shadow tint; black is the usual neutral choice |
| `offset` | Logical pixel displacement; positive X moves right, positive Y moves down |
| `blur` | Edge softness in pixels; larger values produce a broader soft edge |
| `spread` | Expands the shadow silhouette when positive, contracts it when negative |
| `opacity` | Shadow strength from 0.0 to 1.0 |
| `falloff` | How the shadow fades away from its silhouette |

`OuterShadow::new(color)` starts with **zero opacity**, so set `.with_opacity(...)` to make a custom shadow visible. Blur and falloff are shader parameters, not a CSS `box-shadow` declaration; compare the rendered result rather than expecting pixel-identical browser output.

## Falloff

- `ShadowFalloff::Linear`: a linear fade.
- `ShadowFalloff::Smooth`: a smooth fade, used by the presets.
- `ShadowFalloff::Gaussian`: a Gaussian-style fade.

Falloff changes the fade profile, independently of the shadow's offset and spread.

## Runtime Updates

Change `surface.effects.outer_shadow` to update a shadow at runtime:

```rust
use beverly::rendering::{OuterShadow, Surface};
use bevy::prelude::*;

fn set_shadow_strength(surface: &mut Surface, opacity: f32) {
    surface.effects.outer_shadow = Some(
        OuterShadow::regular(Color::BLACK)
            .with_opacity(opacity.clamp(0.0, 1.0)),
    );
}
```

Use `surface.clear_outer_shadow()` to remove it. A surface has one outer-shadow slot; setting another shadow replaces the previous one.

Leave space around the surface for the shadow to render. A clipping ancestor can cut off the effect, and shadows do not reserve space between neighboring elements automatically.

Explicit shadow colors do not change automatically with the theme. Black shadows are less visible on dark backgrounds; use an appropriate surface fill and a subtle [border](borders.md) to maintain separation rather than relying on the shadow alone.

## Demo

Run the interactive drop-shadow demo from the repository root:

```sh
cargo run --example outer_shadow
cargo run --example outer_shadow -- dark
```

The demo includes horizontal and vertical offset sliders, softness, spread, and opacity controls, plus Linear, Smooth, and Gaussian falloff selectors. The main preview uses the selected parameters; the Small, Regular, and Large samples retain their preset shadows for comparison.

The top-right toggle switches light/dark mode. Surface fills and borders follow the theme while the shadows remain black.

## Inner Shadows

`InnerShadow` paints shading **inside** a surface's shape, making it appear recessed rather than elevated. Use it for inset trays, pressed surfaces, and recessed controls. It is a shader-backed surface effect, not a separate child node or a background-blur operation.

```rust
use beverly::rendering::{InnerShadow, Paint, ShadowFalloff, Surface};
use bevy::prelude::*;

fn add_recessed_tray(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        Node {
            width: Val::Px(300.0),
            height: Val::Px(180.0),
            padding: UiRect::all(Val::Px(40.0)),
            ..default()
        },
        Surface::rounded_rect_fill(8.0, Paint::solid(Color::WHITE))
            .inner_shadow(
                InnerShadow::new(Color::BLACK)
                    .with_offset(Vec2::new(0.0, 8.0))
                    .with_blur(16.0)
                    .with_spread(4.0)
                    .with_opacity(0.45)
                    .with_falloff(ShadowFalloff::Smooth),
            ),
    ));
}
```

`BeverlyPlugin` includes the renderer required for this example. The shading follows the surface's rounded corners; it does not change the node's size or reserve padding for its contents.

### Inset Parameters

The inner-shadow API uses the same parameter names as `OuterShadow`:

| Setter | Purpose |
| --- | --- |
| `with_offset(Vec2)` | Displace the shadow in logical UI pixels; X points right and Y points down |
| `with_blur(pixels)` | Adjust the softness of the inset shading |
| `with_spread(pixels)` | Adjust the inset shadow's spread; signed values are supported |
| `with_opacity(value)` | Set strength from 0.0 to 1.0 |
| `with_falloff(mode)` | Choose Linear, Smooth, or Gaussian fade |

The constructor's color sets the shadow tint. `InnerShadow::new(color)` defaults to **zero opacity**, so set an opacity explicitly or use a preset. Compare inset spread visually rather than assuming it expands like an outer shadow: the shading is constrained to the inside of the shape.

### Inset Presets

All three presets use zero spread and smooth falloff:

| Constructor | Offset X / Y | Blur | Opacity |
| --- | --- | --- | --- |
| `InnerShadow::small(color)` | 0 / 1 px | 2 px | 0.075 |
| `InnerShadow::regular(color)` | 0 / 2 px | 4 px | 0.125 |
| `InnerShadow::large(color)` | 0 / 4 px | 8 px | 0.175 |

```rust
use beverly::rendering::{InnerShadow, Paint, Surface};
use bevy::prelude::*;

let surface = Surface::rounded_rect_fill(8.0, Paint::solid(Color::WHITE))
    .inner_shadow(InnerShadow::regular(Color::BLACK));
```

### Updating and Combining

Update `surface.effects.inner_shadow` at runtime, or call `surface.clear_inner_shadow()` to remove it. Each surface has one inner-shadow slot; a new value replaces the existing inset shadow.

Inner and outer shadows occupy separate slots, so a surface can use both. Borders and fills remain independently configurable. Explicit shadow colors are not automatically replaced when the theme changes.

Keep text clear of heavy edge shading with layout padding. In dark mode, black inset shading can become subtle; preserve readable surface contrast and do not use the shadow as the only indication of focus, selection, or disabled state.

### Inner Shadow Demo

Run the interactive inset-shadow demo from the repository root:

```sh
cargo run --example inner_shadow
cargo run --example inner_shadow -- dark
```

The demo includes horizontal and vertical offsets, softness, spread, and opacity controls, plus three falloff selectors. The main recessed tray follows these settings while Small, Regular, and Large samples retain their preset shadows.

The top-right toggle switches light/dark mode. Surface fills and borders follow the theme while the shadows remain black.

## Common uses

- cards and panels
- modal shells and overlays
- hover states on floating controls
- tooltips and context menus
- data surfaces that need a clear sense of separation from the background

## Good shadow behavior

A strong shadow system should be:

- consistent across the app
- tuned to the actual elevation scale
- subtle enough to preserve readability
- distinct enough to communicate hierarchy
- easy to adjust for dark and light themes

The goal is not to create heavy, dramatic shadows everywhere. It is to establish a clear system of depth that feels calm and intentional.

## Avoiding over-shadowing

Too many strong shadows make the interface feel noisy and visually crowded. If every card and panel has a large visible shadow, the hierarchy becomes muddy and the app loses calmness.

This is why Beverly should favor a small set of preset values rather than a free-form shadow system that encourages each component to tune itself independently. A restrained preset model makes the product feel coherent.

## Shadows and accessibility

Shadows are not a replacement for contrast or focus states. They should support a surface, not hide a problem. In a high-contrast or reduced-visual-noise environment, a strong shadow may become too heavy or unnecessary.

That means shadow intensity should be part of the design policy layer, not just a static decoration value. The system should be able to reduce or suppress certain effects when accessibility or theme constraints require it.

## The principle

Use a single shadow system with a small set of presets so the app stays coherent across components. Pick the lightest shadow that still communicates the right depth. The best shadows are the ones users do not consciously notice because they make the structure clearer without interrupting the content.
