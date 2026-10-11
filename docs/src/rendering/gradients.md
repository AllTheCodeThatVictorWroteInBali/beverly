# Gradients

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

Gradients add polish and visual hierarchy to surfaces without requiring custom widget logic. They are often used for hero panels, accent areas, and interactive emphasis. In a design system, gradients are best treated as a controlled surface tool rather than as a free-form visual flourish layered onto every card.

## What gradients do well

A gradient can help the interface do several useful things:

- establish brand tone in a hero or shell surface
- create subtle emphasis without relying on a harsh solid fill
- signal focus or strong state without creating an overly loud border
- add a premium feel to overlays, panels, or onboarding surfaces

They are especially effective when used in limited, purposeful places. A gradient can help define a visual hierarchy, but it should not replace structure, spacing, or typography as the source of meaning.

## Example

```rust
use beverly::rendering::{GradientStop, LinearGradient, Paint, Surface};
use bevy::prelude::*;

fn build_gradient_panel(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        Node {
            width: Val::Px(240.0),
            height: Val::Px(240.0),
            ..default()
        },
        Surface::rounded_rect_fill(
            8.0,
            Paint::linear(LinearGradient::angle_degrees(45.0, vec![
                GradientStop::new(0.0, Color::srgb(0.08, 0.62, 0.42)),
                GradientStop::new(0.5, Color::srgb(0.98, 0.82, 0.20)),
                GradientStop::new(1.0, Color::srgb(0.94, 0.28, 0.34)),
            ])),
        ),
    ));
}
```

`BeverlyPlugin` installs the surface renderer. The node supplies dimensions; `Surface` supplies the rounded shape and gradient paint. No custom shader registration is needed.

## Linear Gradients

Linear gradients blend colors along a line. Use `LinearGradient::horizontal(stops)`, `vertical(stops)`, `angle_degrees(angle, stops)`, or `angle_radians(angle, stops)`.

For explicit endpoints, use `LinearGradient::new(start, end, stops)`. Coordinates are normalized to the surface: `(0, 0)` is top-left and `(1, 1)` is bottom-right.

The angle constructors use `0` degrees for left-to-right and `90` degrees for top-to-bottom. Positive angles follow the UI's downward Y axis. On nonsquare surfaces, normalized coordinates can stretch the visual direction.

## Radial Gradients

A radial gradient blends outward from a center. `radius` is a normalized `Vec2`, allowing different horizontal and vertical extents:

```rust
use beverly::rendering::{GradientStop, Paint, RadialGradient, Surface};
use bevy::prelude::*;

let radial = Surface::rounded_rect_fill(
    8.0,
    Paint::radial(RadialGradient::new(
        Vec2::splat(0.5),
        Vec2::new(0.65, 0.45),
        vec![
            GradientStop::new(0.0, Color::srgb(0.98, 0.82, 0.20)),
            GradientStop::new(1.0, Color::srgb(0.08, 0.62, 0.42)),
        ],
    )),
);
```

`RadialGradient::circular(center, radius, stops)` sets equal normalized radii on both axes. It looks circular on a square surface; unequal surface dimensions stretch it. Smaller radii concentrate the transition closer to the center.

## Angular Gradients

Angular gradients blend around a center over one full turn:

```rust
use beverly::rendering::{AngularGradient, GradientStop, Paint, Surface};
use bevy::prelude::*;

let angular = Surface::rounded_rect_fill(
    8.0,
    Paint::angular(AngularGradient::angle_degrees(
        Vec2::splat(0.5),
        45.0,
        vec![
            GradientStop::new(0.0, Color::srgb(0.12, 0.55, 0.90)),
            GradientStop::new(0.5, Color::srgb(0.94, 0.28, 0.34)),
            GradientStop::new(1.0, Color::srgb(0.12, 0.55, 0.90)),
        ],
    )),
);
```

`AngularGradient::new(center, angle_radians, stops)` accepts radians directly. Zero points right; positive angles rotate clockwise. Equal first and last colors give a continuous wrap. Different endpoint colors create a visible boundary at the start of the turn.

## Color Stops

`GradientStop::new(position, color)` places a color along the gradient. Positions range from `0.0` to `1.0`; moving a middle stop changes how much space the neighboring transitions occupy.

- The shader supports up to five stops (`MAX_GRADIENT_STOPS`). Keep authored gradients within that limit.
- Normalization sorts stops and clamps positions to the valid range. Equal positions retain the last supplied stop.
- Color alpha controls transparency at each stop. The resulting fill blends with the content behind the surface, not with child text or icons.
- Dithering is enabled by default to reduce visible banding. Use `.with_dithering(false)` on a gradient to compare the result without it.

Gradient paint can also be used for a [border](borders.md), independently of the surface fill.

## Runtime Updates

Replace `Surface::fill` with a new `Paint::linear`, `Paint::radial`, or `Paint::angular` in an application system to update the gradient. These gradient types are static until their parameters change; they do not automatically rotate or animate.

Explicit stop colors are not replaced when the theme changes. Build stops from `ThemeResource` when a gradient should follow theme tokens. For animated parameters, honor reduced-motion preferences in the system driving them.

## Demo

Run the interactive gradient demo from the repository root:

```sh
cargo run --example gradients
cargo run --example gradients -- dark
```

The demo shows linear, radial, and angular previews with Fresh, Seaside, and Mono palette swatches. The angle slider controls linear direction and angular rotation; the middle-stop slider changes all three gradients; radial size changes only the radial preview. The previews use square surfaces so radial geometry is easy to compare.

The top-right toggle switches the surrounding page between light and dark mode while the chosen palette remains unchanged.

## Animated borders

`Surface::animated_border` paints the border with a gradient that spins around the surface. It is added like any other surface treatment, and the animation runs on the GPU clock, so it needs no system of your own:

```rust
Surface::rounded_rect_fill(8.0, Color::WHITE)
    .uniform_border(1.0, Color::NONE)
    .animated_border(SpinningGradient::default());
```

The default is a neutral line with a blue and near-white highlight chasing around it. Pass your own stops to change the colors. The speed is fixed, and up to five stops are used:

```rust
Surface::rounded_rect_fill(8.0, Color::WHITE).animated_border(SpinningGradient::new([
    GradientStop::new(0.0, Color::srgb(0.9, 0.2, 0.4)),
    GradientStop::new(0.5, Color::srgb(1.0, 0.8, 0.2)),
    GradientStop::new(1.0, Color::srgb(0.9, 0.2, 0.4)),
]));
```

Stops are positions around one full turn. Keep the first and last colors equal so the loop has no seam. An existing border width is kept; otherwise the border is 1px. The spin stops when the user prefers reduced motion.

## Guidance

- use gradients to direct attention, not as a replacement for strong hierarchy
- keep contrast high enough for readable text
- prefer soft, controlled transitions over dramatic color shifts
- keep brand gradients limited to key surfaces rather than spreading them through the entire app
- ensure gradients remain readable under dark, light, and high-contrast modes

## Gradients and content legibility

A gradient is only successful when the surrounding content remains easy to read. A bright, saturated gradient behind dense text can quickly reduce legibility and make the interface harder to use.

This is why Beverly should favor gradients that are:

- soft rather than aggressive
- harmonious with the app’s color system
- easy to pair with text and border contrast
- intentional and limited in scope

## Theme-aware use

Gradients should feel like part of the theme, not as random decoration. The system should use tokens and colors from the product design language so gradients remain consistent across surfaces and screens.

A gradient used in one screen should feel like it belongs to the same visual system as the rest of the product, not like a special effect imported from elsewhere.

## The core idea

Gradients are a surface-level tool for emphasis and atmosphere. They should add quality to the design without introducing visual noise or reducing readability. In a strong design system, they are used purposefully, consistently, and with accessibility in mind.
