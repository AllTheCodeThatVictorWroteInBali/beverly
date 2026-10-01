# Shaders

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

In Beverly, most UI shader work is expressed through a `Surface`, not through a different shader for every widget. A `Surface` describes a shape and its visual data; Beverly's rendering plugin turns that description into a shared GPU material. Cards, panels, and controls can therefore share one renderer while keeping their own meaning and behavior.

> **Describe the surface in Rust. Let the renderer translate it for the GPU.**

## The Beverly path

The standard application setup registers Beverly's renderer through `BeverlyPlugin`:

```rust
use bevy::prelude::*;
use beverly::prelude::BeverlyPlugin;
use beverly::rendering::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BeverlyPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    let stops = vec![
        GradientStop::new(0.0, Color::srgb(0.10, 0.38, 0.72)),
        GradientStop::new(1.0, Color::srgb(0.20, 0.72, 0.62)),
    ];

    commands.spawn((
        Node {
            width: px(320.0),
            height: px(120.0),
            ..default()
        },
        Surface::new(
            Shape::rounded_rect(18.0),
            Paint::linear(LinearGradient::horizontal(stops)),
        )
        .uniform_border(1.0, Color::srgba(0.8, 0.9, 1.0, 0.65))
        .outer_shadow(OuterShadow::small(Color::BLACK)),
    ));
}
```

`BeverlyPlugin` installs `UiRenderingPlugin`, registers the material, embeds Beverly's WGSL asset, and creates a default 2D UI camera when the app has not supplied one. Add a Bevy UI `Node` to give the entity layout and size; add a `Surface` to give it shader-rendered appearance. The example uses only the public surface API: a rounded rectangle, a horizontal two-stop gradient, a border, and a small shadow.

For the paint, border, shadow, blur, and glass APIs in more depth, see [Materials](materials.md), [Gradients](gradients.md), [Effects](effects.md), and [Liquid Glass](liquid-glass.md).

## What a `Surface` describes

`Surface` is the renderer-facing description of a UI surface. Its core fields are:

- `shape` — currently a rounded rectangle with either one radius or per-corner radii
- `fill` — solid color, shimmer, or linear, radial, or angular gradient paint
- `border` — optional border paint and widths
- `decorations` — semantic decoration such as a focus ring
- `effects` — optional outer shadow, outer glow, or inner shadow
- `noise`, `clip`, `mask`, and `backdrop` — optional surface treatments

These are data, not independent widget render paths. For example, changing the fill does not change the entity's layout or turn a card into a different component. It changes the paint data that the renderer sends to the shader.

## From layout to fragment

The rendering flow keeps responsibilities separate:

1. Bevy UI computes the entity's layout, size, and transform.
2. Beverly reads the entity's `Surface` and the computed UI geometry.
3. Beverly encodes shape, paint, border, effect, and backdrop values into the material uniform.
4. The UI material binds that data and the embedded WGSL evaluates the surface for each fragment.

This is why a `Surface` should be attached to an entity that participates in Bevy UI layout. The renderer needs the computed size and transform to evaluate its local shape, place the effects, and sample a backdrop correctly.

The GPU material is managed by `UiRenderingPlugin`. Applications should update the `Surface` component rather than creating or editing Beverly's internal `UiShapeMaterial` directly. The renderer handles material synchronization as the surface data or computed layout changes.

## Paint and geometry

Paint is independent from shape. The same fill can be applied to different rounded-rectangle radii, and a shape can switch from a solid fill to a gradient without changing its geometry.

```rust
let fill = Paint::linear(LinearGradient::new(
    Vec2::new(0.0, 0.0),
    Vec2::new(1.0, 1.0),
    vec![
        GradientStop::new(0.0, Color::srgb(0.12, 0.34, 0.76)),
        GradientStop::new(0.55, Color::srgb(0.18, 0.70, 0.68)),
        GradientStop::new(1.0, Color::srgb(0.82, 0.88, 0.43)),
    ],
));

let surface = Surface::new(Shape::rounded_rect(16.0), fill);
```

Gradient coordinates are normalized local coordinates: `(0, 0)` is the top-left and `(1, 1)` is the bottom-right. Beverly currently encodes up to four gradient stops for the GPU. Stops are normalized and sorted before encoding. Authored colors are converted to linear RGBA for shader evaluation; choose colors using Bevy's color APIs rather than pre-linearizing them yourself.

Rounded corners are evaluated from the surface shape in the shader rather than by changing the widget's layout. That lets the border, fill, and effects use the same shape description. The current shape API supports rounded rectangles, including distinct radii for each corner.

## Effects, focus, and backdrops

Effects and decorations are part of the same surface description, but they serve different purposes. Shadows and glows add depth or emphasis; a focus ring communicates an interaction state. Keep focus visible independently of decorative styling, including when a surface is translucent.

```rust
let elevated = Surface::new(
    Shape::rounded_rect(12.0),
    Color::srgb(0.12, 0.15, 0.20).into(),
)
.outer_shadow(OuterShadow::regular(Color::BLACK));
```

Backdrop blur and liquid-glass treatments also use the shared material, but they depend on the renderer's backdrop capture path. They are not equivalent to making a transparent fill: the renderer must capture and sample the content behind the surface. Use them selectively, and retain a readable fill or contrast treatment so text and controls remain legible.

## Editing or replacing the shader

Beverly's built-in shader is `src/rendering/shaders/ui_shape.wgsl`. Its Rust-side material and uniform definitions live alongside it in `src/rendering/material.rs`; `UiRenderingPlugin` embeds the WGSL and registers `UiShapeMaterial` as a Bevy UI material.

The Rust and WGSL uniform layouts are one contract. A change to a field's type, order, alignment, or meaning must be made consistently on both sides. The shader also relies on the UI material's vertex inputs and the backdrop texture and sampler bindings. Treat those bindings as part of the interface, not as incidental implementation details.

Most application styling does not need a custom shader: use `Surface` and its public paint/effect types. Consider a separate Bevy `UiMaterial` when the effect requires a different data model, bindings, or rendering behavior that the `Surface` contract does not represent. Keep that material and shader in the application or extension that owns the effect instead of coupling ordinary widgets to Beverly's internal material type.

## Debugging

Beverly includes renderer debug views for inspecting the rounded-rectangle distance field, border coverage and paint, gradient coordinates, noise, and focus-ring coverage. Select one with `UI_RENDER_DEBUG` when launching the app:

```sh
UI_RENDER_DEBUG=gradient_uv cargo run
```

Accepted values include `sdf`, `border`, `gradient_uv`, `border_paint`, `noise`, `noise_coords`, `noise_strength`, `noise_modulation`, `focus`, and `focus_distance`. The default is the final rendered surface. For a built-in rendering sample, run an application with `UI_RENDERING_DEMO=1`; backdrop diagnostics are available through `UI_BACKDROP_DEBUG` (`source`, `blur`, or `mask`), and backdrop rendering can be disabled with `UI_BACKDROP_DISABLED=1`.

When a surface is missing or malformed, check these in order:

1. Confirm the entity has both a sized Bevy UI `Node` and a `Surface`.
2. Confirm the app added `BeverlyPlugin` (or registered `UiRenderingPlugin` directly).
3. Reduce the effect to a solid fill and rounded rectangle, then add paint and effects back one at a time.
4. Use the matching debug view to distinguish bad geometry or uniform data from a paint or effect issue.
5. Check the application log for shader compilation, asset, or render-pipeline diagnostics.

## Performance and accessibility

The shared renderer avoids introducing a separate widget-specific shader path for each visual variant, but effects still have costs. In particular, backdrop sampling and large blur, glow, or shadow regions deserve attention on screens with many surfaces. Prefer a small number of intentional effects over applying the most expensive treatment to every row, chip, or button.

Keep the visual state understandable without relying on shader effects alone. Text contrast, selected and disabled states, and keyboard focus must remain clear if blur, noise, or transparency is reduced or unavailable. Beverly's runtime effect policy can suppress selected effects; application content should still communicate its meaning through structure, labels, and state.

> **Use shaders to express a surface, not to hide the interface's structure.**
