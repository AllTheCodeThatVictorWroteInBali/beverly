# Liquid Glass

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

The liquid-glass pattern is the most advanced visual treatment in a modern UI system. It adds translucency, refraction, highlight response, and a distinctive layered feel while preserving readability. This is the style most closely associated with theblocks_studio reference work: a single Surface model is given a glass-like material, a blur treatment, and optical highlight calculations rather than a one-off hack.

## Why liquid glass is special

Liquid glass is not just a transparency effect. It is a compositional surface treatment that tries to create a premium, layered, and atmospheric effect while still making the interface readable and usable. That makes it a powerful design tool when applied intentionally.

It works well for:

- modal shells and negotiation surfaces
- floating toolbars and utility panels
- premium overlays and command surfaces
- glass-like cards that need visual differentiation

The design goal is to create a feeling of depth and softness without losing the structure or legibility of the content inside the surface.

## Usage

Liquid glass is the optical layer of a [backdrop](blur.md). Attach a `Backdrop` that carries a `LiquidGlass` to a `Surface`; `BeverlyPlugin` installs the renderer and capture pass.

```rust
use beverly::rendering::{
    Backdrop, BackdropQuality, GlassProfile, LiquidGlass, Paint, Surface,
};
use bevy::prelude::*;

fn add_glass_lens(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        Node {
            width: Val::Px(300.0),
            height: Val::Px(140.0),
            ..default()
        },
        Surface::rounded_rect_fill(70.0, Paint::solid(Color::NONE))
            .uniform_border(1.0, Paint::solid(Color::WHITE.with_alpha(0.5)))
            .with_backdrop(
                Backdrop::new()
                    .with_blur(2.0)
                    .with_tint(Color::WHITE)
                    .with_tint_opacity(0.06)
                    .with_saturation(1.15)
                    .with_quality(BackdropQuality::High)
                    .with_liquid_glass(LiquidGlass {
                        thickness: 8.0,
                        bezel_width: 16.0,
                        refractive_index: 1.46,
                        profile: GlassProfile::Squircle,
                        ..default()
                    }),
            ),
    ));
}
```

The glass follows the surface's shape. Geometry, fill, border, and shadows stay on the `Surface`; blur, tint, and saturation stay on the `Backdrop`; only the optics live in `LiquidGlass`. The example uses a transparent fill so the glass is the visible material.

Like any backdrop surface, it can only bend content drawn **before** it. See [what gets blurred](blur.md#what-gets-blurred) for the capture rules: one shared snapshot, no glass-on-glass, and no SVG icons.

## Optical Parameters

`LiquidGlass::default()` is enabled and starts with a subtle lens. Set any field with struct update syntax (`..default()`).

| Field | Default | Valid range | Effect |
| --- | --- | --- | --- |
| `enabled` | `true` | | Turns the optical layer on or off |
| `thickness` | 3.0 px | 0 to 24 | How strongly the edge bends the backdrop |
| `bezel_width` | 4.5 px | 0.5 to 48 | Width of the curved edge band; the center stays flat |
| `refractive_index` | 1.46 | 1.0 to 1.8 | Bending strength; `1.0` bends nothing |
| `specular_intensity` | 0.28 | 0 to 0.6 | Brightness of the directional highlight |
| `specular_width` | 0.8 | 0.1 to 8 | Width of the highlight band |
| `light_direction` | (-0.6, -0.8) | normalized | Where the highlight comes from; the default is the top-left |
| `fresnel` | 0.14 | 0 to 0.4 | Brightening toward steep edges |
| `chromatic_aberration` | 0.018 | 0 to 0.08 | Red/blue color fringing at the edge |
| `profile` | `Squircle` | see below | Shape of the edge curve |
| `press_amount` | 0.0 | 0 to 1 | Pressed-state optical response |

Values outside these ranges are clamped, and non-finite values fall back to their defaults, so an extreme setting cannot break the surface. `thickness` and `bezel_width` scale together: a thick lens with a narrow bezel produces a steep, strongly bent edge.

The bending is bounded to about a third of the bezel width, so large `thickness` values increase the curvature but cannot smear content across the whole surface. Increase `bezel_width` to give the effect more room.

## Profiles

`GlassProfile` selects the cross-section of the edge:

- `Convex`: a rounded dome profile.
- `Squircle`: a softer, squarer dome; the default.
- `Concave`: a recessed, meniscus-like edge.
- `Lip`: a raised rim that blends the convex and concave curves.

Try each against a background with straight lines to compare how they bend them.

## Pairing with Backdrop

The glass uses the backdrop's blur, tint, saturation, brightness, and contrast. When glass is enabled, the blur is slightly softer at the bezel than in the center, so keep `with_blur` small (a few pixels) for a clear lens. A heavier blur gives a frosted look instead; see [Blur](blur.md).

`BackdropQuality::High` takes the most blur samples and gives the smoothest result for a surface that is large or sits over high-contrast content.

## Press Response

`press_amount` is a normalized value you drive from your own interaction code, for example from a button's pressed state. It adjusts only the optical slope and shift; it never resizes the surface, its fill, its hit area, or its children.

```rust
use beverly::rendering::Surface;
use bevy::prelude::*;

fn set_press(surface: &mut Surface, pressed: bool) {
    if let Some(backdrop) = surface.backdrop.as_mut() {
        if let Some(glass) = backdrop.liquid_glass.as_mut() {
            glass.press_amount = if pressed { 1.0 } else { 0.0 };
        }
    }
}
```

The press response is suppressed when the user prefers reduced motion.

## Updating at Runtime

Replace `surface.backdrop` to change any parameter, or call `surface.clear_backdrop()` to remove the effect. The demo assigns a new backdrop from its sliders and skips the assignment when nothing changed.

## Accessibility and Fallbacks

Beverly does not automatically replace glass with an opaque surface when reduced transparency or high contrast is requested. Only the press response is suppressed under reduced motion. Apps should check the user's preferences themselves and remove the backdrop in favor of a solid fill and a clear border when needed.

The refraction can distort text drawn behind the glass. Keep important content inside the glass surface rather than relying on content behind it, and do not use glass to hide a focus state.

## Demo

Run the interactive liquid-glass demo from the repository root:

```sh
cargo run --example liquid_glass
cargo run --example liquid_glass -- dark
```

The demo places a pill-shaped and a circular lens over a moving, striped background with fine lines and large text. Five sliders control thickness (0–24 px), bezel width (1–48 px), refraction (100–180%), specular intensity (0–60%), and color fringe (0–0.08); four buttons select the profile. All controls drive both lenses, and the maximum of each slider maps to the top of its valid range.

The background stays dark in both themes so the optics read consistently. The top-right toggle switches the page chrome between light and dark, and the stripes hold still when reduced motion is enabled.

## Considerations

- avoid using glass on dense text surfaces unless contrast remains acceptable
- fall back to opaque or semi-opaque surfaces under high contrast or reduced transparency
- keep the effect consistent across window sizes and theme changes
- prefer a shared material pipeline over many custom glass widgets
- ensure the effect does not hide focus states or critical affordances

## Performance note

Glass is visually rich but more expensive than plain fill and shadow. Use it on key surfaces such as modal shells, floating toolbars, or premium overlays, not on every single widget. This keeps the effect expressive while preventing the GPU cost from becoming a systemic problem.

## Accessibility and policy boundaries

A glass surface should never be allowed to reduce usability. That means the system should respect policies such as:

- reduced transparency
- high-contrast mode
- reduced-motion preferences
- strong focus visibility
- readable text at all times

If the interface needs to fall back to a more solid treatment in those modes, it should do so without creating a separate custom version of the component. The material system should handle the variation.

## The right balance

Liquid glass is best when it sits in a restrained, premium layer. It adds atmosphere, but it should not become a dominant visual element across the whole product. A single carefully chosen glass panel can feel luxurious; many of them can become visually noisy and expensive.

This is exactly why the surface model matters: it gives the framework a disciplined way to apply glass treatments only where they make sense. The effect stays expressive, but the product remains coherent and usable.

## The key idea

Liquid glass is a strong example of design expression built on top of a disciplined material system. It is visually rich, but it only works well when the rest of the product remains structurally clear. That is the difference between a premium product surface and a distracting aesthetic layer.
