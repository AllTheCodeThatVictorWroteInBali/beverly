# Background Color

Background fills are the starting point for Beverly's shader-backed surfaces. Use `Surface` with `Paint::solid` for a flat color, adjust its alpha for transparency, and overlap surfaces to compose layers.

`BeverlyPlugin` includes `UiRenderingPlugin`, which renders these surfaces. No custom shader or material registration is needed for the examples below.

## Solid Fill

Attach a `Surface` to a Bevy UI `Node`. The node supplies the size and layout; the surface supplies the shape and paint.

```rust
use beverly::rendering::{Paint, Surface};
use bevy::prelude::*;

fn add_background(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        Node {
            width: Val::Px(240.0),
            height: Val::Px(192.0),
            ..default()
        },
        Surface::rounded_rect_fill(
            0.0,
            Paint::solid(Color::srgb(0.12, 0.68, 0.48)),
        ),
    ));
}
```

The first argument is the corner radius in pixels. `0.0` gives square corners. `Color::srgb` creates an opaque color with RGB channels from `0.0` to `1.0`.

Use the surface as the visible fill instead of adding a second colored `BackgroundColor` underneath it. For an ordinary Bevy background without Beverly's material effects, `BackgroundColor` remains available.

## Transparency

Set the paint color's alpha to control how much of the content behind it shows through:

```rust
use beverly::rendering::{Paint, Surface};
use bevy::prelude::*;

let mint = Color::srgb(0.12, 0.68, 0.48);
let translucent = Surface::rounded_rect_fill(
    0.0,
    Paint::solid(mint.with_alpha(0.65)),
);
```

- Alpha `1.0`: fully opaque.
- Alpha `0.0`: fully transparent.
- Values between them blend the fill with what is behind it.

This changes the **fill**, not the opacity of the entire UI subtree. Child text, icons, borders, and other effects retain their own colors and opacity. A transparent node also retains its layout and interaction behavior.

Transparency does not blur or distort the background. Those effects belong to [Backdrop Blur](blur.md) and [Liquid Glass](liquid-glass.md).

## Layered Surfaces

Place overlapping surfaces inside a shared layout parent. Later siblings normally draw above earlier ones; use `ZIndex` when explicit ordering is needed.

```rust
use beverly::rendering::{Paint, Surface};
use bevy::prelude::*;

fn add_layers(parent: &mut ChildSpawnerCommands) {
    parent.spawn(Node {
        width: Val::Px(240.0),
        height: Val::Px(192.0),
        position_type: PositionType::Relative,
        ..default()
    }).with_children(|layers| {
        layers.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            Surface::rounded_rect_fill(
                0.0,
                Paint::solid(Color::srgb(0.94, 0.30, 0.34)),
            ),
        ));
        layers.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(48.0),
                top: Val::Px(36.0),
                width: Val::Px(144.0),
                height: Val::Px(120.0),
                ..default()
            },
            Surface::rounded_rect_fill(
                0.0,
                Paint::solid(Color::srgb(0.12, 0.68, 0.48).with_alpha(0.65)),
            ),
        ));
    });
}
```

A checkerboard behind a translucent surface makes its alpha easier to inspect. The demo uses this pattern for its transparent and layered previews.

## Updating Colors

Change `Surface::fill` in an application system to update a material at runtime:

```rust
use beverly::rendering::{Paint, Surface};
use beverly::theme::ThemeResource;
use bevy::prelude::*;

#[derive(Component)]
struct ThemeBackground;

fn update_backgrounds(
    theme: Res<ThemeResource>,
    mut surfaces: Query<&mut Surface, With<ThemeBackground>>,
) {
    let fill = Paint::solid(theme.current.colors.background);
    for mut surface in &mut surfaces {
        if surface.fill != fill {
            surface.fill = fill.clone();
        }
    }
}
```

Register this system in `Update` and attach `ThemeBackground` to the surfaces it should control. An explicitly supplied color is not automatically replaced when light/dark mode changes; read `ThemeResource` when a fill should follow the theme.

Check text contrast against the final blended background, not just the unblended fill color.

## Demo

Run the background-color demo from the repository root:

```sh
cargo run --example background_color
cargo run --example background_color -- dark
```

The demo includes five color swatches, a 0–100% opacity slider, and three previews: solid fill, transparency over a checkerboard, and overlapping colored surfaces. The solid preview remains opaque while the other two follow the opacity control. The top-right button switches the page between light and dark mode.