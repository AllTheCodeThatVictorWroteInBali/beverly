# Rounded Corners

Beverly renders rounded corners through the `Shape` attached to a `Surface`. Corner radii are measured in logical UI pixels; the Bevy `Node` supplies the surface's dimensions.

`BeverlyPlugin` installs the renderer needed for these examples.

## Uniform Radius

Pass a radius to `Surface::rounded_rect_fill` to round all four corners equally:

```rust
use beverly::rendering::{Paint, Surface};
use bevy::prelude::*;

fn add_rounded_surface(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        Node {
            width: Val::Px(240.0),
            height: Val::Px(120.0),
            ..default()
        },
        Surface::rounded_rect_fill(
            16.0,
            Paint::solid(Color::srgb(0.10, 0.48, 0.34)),
        ),
    ));
}
```

A radius of `0.0` produces square corners. `Shape::rounded_rect(radius)` provides the same shape for `Surface::new` or `.with_shape(...)`.

## Independent Corners

`Shape::rounded_rect_corners` accepts four radii in clockwise order: **top left, top right, bottom right, bottom left**.

```rust
use beverly::rendering::{Paint, Shape, Surface};
use bevy::prelude::*;

let surface = Surface::new(
    Shape::rounded_rect_corners(0.0, 24.0, 48.0, 12.0),
    Paint::solid(Color::srgb(0.72, 0.20, 0.26)),
);
```

Use `0.0` for any corner that should remain square. The corner order differs from [border widths](borders.md), which use top/right/bottom/left sides.

For direct data access, `CornerRadii` exposes `top_left`, `top_right`, `bottom_right`, and `bottom_left`. The uniform constructor clamps negative values to zero; independent corner constructors store their arguments directly. Supply nonnegative radii or sanitize them with `CornerRadii::clamped_non_negative()`.

## Pills and Circles

A pill uses a radius equal to half its shorter dimension. A circle additionally requires equal width and height.

```rust
use beverly::rendering::{Paint, Surface};
use bevy::prelude::*;

fn add_pill_and_circle(parent: &mut ChildSpawnerCommands) {
    let fill = Paint::solid(Color::srgb(0.12, 0.40, 0.72));
    parent.spawn((
        Node {
            width: Val::Px(240.0),
            height: Val::Px(80.0),
            ..default()
        },
        Surface::rounded_rect_fill(40.0, fill.clone()),
    ));
    parent.spawn((
        Node {
            width: Val::Px(120.0),
            height: Val::Px(120.0),
            ..default()
        },
        Surface::rounded_rect_fill(60.0, fill),
    ));
}
```

Changing a square into a wider rectangle while retaining the same large radius creates a pill rather than a circle. Keep dimensions stable when the shape must remain circular.

## Borders and Content

A painted border follows the surface shape:

```rust
use beverly::rendering::{Paint, Surface};
use bevy::prelude::*;

let surface = Surface::rounded_rect_fill(16.0, Paint::solid(Color::WHITE))
    .uniform_border(2.0, Paint::solid(Color::srgb(0.5, 0.5, 0.5)));
```

The shader's radius is separate from Bevy's `Node::border_radius`. Changing only the node's radius does not replace `Surface::shape`. Rounded surface paint also does not automatically clip child text, images, or icons; configure content clipping separately when required.

Leave sufficient padding so content stays clear of curved corners, especially inside pills and circles.

## Resizing and Animation

Update `Surface::shape` to animate the radius. When a circular element changes size, set the radius to half its current size on every animation frame. Any explicitly configured mask should use the same shape.

```rust
use beverly::rendering::{Shape, Surface};
use bevy::prelude::*;

fn resize_circle(node: &mut Node, surface: &mut Surface, size: f32) {
    let size = size.max(0.0);
    node.width = Val::Px(size);
    node.height = Val::Px(size);
    surface.shape = Shape::rounded_rect(size * 0.5);
    if let Some(mask) = surface.mask.as_mut() {
        mask.shape = surface.shape;
    }
}
```

Recalculate positioning as well when a growing element must remain centered. Keeping the old radius or top offset while enlarging a handle can produce a rounded square or an off-center circle.

## Demo

Run the rounded-corners demo from the repository root:

```sh
cargo run --example rounded_corners
cargo run --example rounded_corners -- dark
```

The demo includes a uniform-radius slider and four independent-corner sliders, each ranging from 0 to 60 pixels. Fixed-size previews show uniform corners, independent corners, a pill, and a circle. The pill and circle retain a 60-pixel radius and 120-pixel height while the other previews follow the sliders.

The top-right toggle switches the surrounding page between light and dark mode.