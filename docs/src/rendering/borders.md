# Borders

Beverly's `Surface` renderer paints borders around a surface's shape. A border combines widths in pixels with a `Paint`, which can be a solid color or a gradient.

`BeverlyPlugin` includes the rendering plugin required for these examples. For shared design values rather than rendering APIs, see [Border Tokens](../foundations/borders.md).

## Uniform Border

Use `uniform_border` to give every side the same width and paint:

```rust
use beverly::rendering::{Paint, Surface};
use bevy::prelude::*;

fn add_bordered_surface(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        Node {
            width: Val::Px(240.0),
            height: Val::Px(120.0),
            padding: UiRect::all(Val::Px(20.0)),
            ..default()
        },
        Surface::rounded_rect_fill(8.0, Paint::solid(Color::WHITE))
            .uniform_border(
                4.0,
                Paint::solid(Color::srgb(0.12, 0.68, 0.48)),
            ),
    ));
}
```

The surface's shape determines its corners; border thickness does not set the corner radius. A width of `0.0` makes the border invisible.

The equivalent explicit configuration is `.border(Border::new(width, paint))`.

## Individual Sides

Use `Border::per_side` with `BorderWidths::sides`. The argument order is **top, right, bottom, left**.

```rust
use beverly::rendering::{Border, BorderWidths, Paint, Surface};
use bevy::prelude::*;

let underline = Surface::rounded_rect_fill(0.0, Paint::solid(Color::WHITE))
    .border(Border::per_side(
        BorderWidths::sides(0.0, 0.0, 4.0, 0.0),
        Paint::solid(Color::srgb(0.5, 0.5, 0.5)),
    ));

let asymmetric = Surface::rounded_rect_fill(8.0, Paint::solid(Color::WHITE))
    .border(Border::per_side(
        BorderWidths::sides(2.0, 4.0, 6.0, 8.0),
        Paint::solid(Color::srgb(0.12, 0.55, 0.90)),
    ));
```

Other width constructors include:

| Constructor | Result |
| --- | --- |
| `BorderWidths::all(width)` | Same width on every side |
| `BorderWidths::symmetric(vertical, horizontal)` | Top/bottom use `vertical`; left/right use `horizontal` |
| `BorderWidths::horizontal_vertical(horizontal, vertical)` | Same pairing, with reversed argument order |
| `BorderWidths::ZERO` | No border on any side |

The uniform and symmetric constructors clamp negative widths to zero. `sides` stores its arguments directly; use nonnegative widths or call `.clamped_non_negative()`.

## Gradient Border

Border paint is independent of fill paint. A surface can have a solid background and a gradient outline:

```rust
use beverly::rendering::{GradientStop, LinearGradient, Paint, Surface};
use bevy::prelude::*;

let surface = Surface::rounded_rect_fill(8.0, Paint::solid(Color::WHITE))
    .uniform_border(
        4.0,
        Paint::linear(LinearGradient::horizontal(vec![
            GradientStop::new(0.0, Color::srgb(0.12, 0.68, 0.48)),
            GradientStop::new(1.0, Color::srgb(0.94, 0.30, 0.34)),
        ])),
    );
```

This paints a continuous gradient around the outline. It does not assign a separate color to each side. See [Gradients](gradients.md) for additional paint types.

For a translucent solid border, use `Paint::solid(color.with_alpha(opacity))`. Border opacity does not change the surface fill or its child content.

## Layout and Theme

`Surface::border` describes the painted border. Bevy's `Node::border` describes layout border widths. Configure layout spacing explicitly when content needs room beside a thick outline; the demo uses padding to keep its labels clear of the painted edges.

Do not add a second visible `BorderColor` unless you intend to draw a separate Bevy border as well.

Explicit border colors remain unchanged when the theme switches. To follow the theme, read `ThemeResource` and update the border paint:

```rust
use beverly::rendering::{Paint, Surface};
use beverly::theme::ThemeResource;
use bevy::prelude::*;

#[derive(Component)]
struct ThemedOutline;

fn update_outlines(
    theme: Res<ThemeResource>,
    mut surfaces: Query<&mut Surface, With<ThemedOutline>>,
) {
    let paint = Paint::solid(theme.current.colors.border);
    for mut surface in &mut surfaces {
        if let Some(border) = surface.border.as_mut() {
            if border.paint != paint {
                border.paint = paint.clone();
            }
        }
    }
}
```

Register this system in `Update` and attach `ThemedOutline` to the relevant surfaces. Use `surface.clear_border()` to remove a border entirely. Border setters replace the existing border rather than adding another outline.

## Demo

Run the interactive border demo from the repository root:

```sh
cargo run --example borders
cargo run --example borders -- dark
```

The demo includes four color swatches and a 0–16 pixel thickness slider. Its previews show uniform, bottom-only, asymmetric, and gradient borders. The asymmetric widths are half, one, one-and-a-half, and twice the slider value, in top/right/bottom/left order.

The gradient runs from the selected color to coral. Preview fills follow the active theme, and the top-right button switches between light and dark mode.