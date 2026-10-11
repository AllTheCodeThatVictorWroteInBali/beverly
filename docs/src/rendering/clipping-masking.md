# Clipping and Masking

`Clip` and `Mask` trim or fade a `Surface`'s own paint with a rounded-rectangle shape. Both are applied by the surface shader, so they follow the surface's rounded corners and antialiasing. `BeverlyPlugin` installs the renderer needed for the examples below.

They affect the **surface**, not its child nodes. Neither one clips child text, images, or icons. For that, use Bevy's `Node::overflow` on the parent.

## Clip

```rust
use beverly::rendering::{Clip, GradientStop, LinearGradient, Paint, Surface};
use bevy::prelude::*;

fn add_clipped_surface(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        Node {
            width: Val::Px(110.0),
            height: Val::Px(100.0),
            ..default()
        },
        Surface::rounded_rect_fill(
            0.0,
            Paint::linear(LinearGradient::angle_degrees(45.0, vec![
                GradientStop::new(0.0, Color::srgb(0.98, 0.76, 0.18)),
                GradientStop::new(1.0, Color::srgb(0.94, 0.26, 0.32)),
            ])),
        )
        .with_clip(Clip::rounded_rect(28.0)),
    ));
}
```

`Clip::rounded_rect(radius)` rounds the corners of everything the surface paints. Use `Clip::new(shape)` with a `Shape` for independent corner radii; see [Rounded Corners](rounded-corners.md).

The clip box is the surface's own box. It cannot be larger or offset, so changing its radius only trims the corners; it does not reveal or crop a different region.

## Mask

```rust
use beverly::rendering::{Mask, Paint, Shape, Surface};
use bevy::prelude::*;

let masked = Surface::rounded_rect_fill(0.0, Paint::solid(Color::srgb(0.12, 0.72, 0.50)))
    .with_mask(Mask::new(Shape::rounded_rect(28.0)).with_opacity(0.6));
```

A mask has the same shape and radius controls as a clip. Its opacity multiplies the coverage, so `with_opacity(0.6)` fades the painted surface to 60 percent. `Mask::rounded_rect(radius)` is a shorthand.

## What They Affect

Clip and mask are multiplied together, then applied to the surface's:

| Layer | Trimmed by clip and mask |
| --- | --- |
| Fill, including gradients | Yes |
| Border | Yes |
| Inner shadow | Yes |
| Backdrop blur and [liquid glass](liquid-glass.md) | Yes |
| [Outer shadow](shadows.md) | **No** |
| [Outer glow](glow.md) | **No** |
| Child nodes (text, icons, images) | **No** |
| Focus ring | Clip only |

Outer shadows and glows are drawn around the surface's own shape and are not trimmed, so a clipped surface keeps a shadow shaped by its original corners. If you need the shadow to follow a larger radius, change the surface's `Shape` as well. The demo's "Clip + shadow" preview shows this.

A `Clip` also gates the focus ring. A `Mask` does not, so a faded surface keeps a full-strength focus ring.

## Opacity

Both types accept an opacity from 0.0 to 1.0:

```rust
use beverly::rendering::{Clip, Mask};

let clip = Clip::rounded_rect(16.0).with_opacity(0.5);
let mask = Mask::rounded_rect(16.0).with_opacity(0.5);
```

Out-of-range values are clamped, and non-finite values fall back to `1.0`. A mask or clip at opacity `0.0` hides the surface's own paint entirely, which leaves only an outer shadow or glow.

Use a mask when the point is fading, for example during an exit animation. Use a clip when the point is trimming the shape, particularly if the surface's focus ring should follow it.

## Radius Limits

Radii are limited to half of the surface's shorter side, so an oversized radius produces a pill or circle rather than an invalid shape. Negative radii are treated as zero. A square surface with a radius of at least half its side becomes a circle.

## Updating at Runtime

Assign a new value to the surface field, or use the clear methods:

```rust
use beverly::rendering::{Clip, Mask, Surface};

fn set_cut(surface: &mut Surface, radius: f32, fade: f32) {
    surface.clip = Some(Clip::rounded_rect(radius));
    surface.mask = Some(Mask::rounded_rect(radius).with_opacity(fade));
}

fn remove_cut(surface: Surface) -> Surface {
    surface.clear_clip().clear_mask()
}
```

A surface has one clip and one mask, so assigning a new value replaces the old one.

## Fading Content

A mask fades only the surface's own paint. To fade a whole component, such as a toast, also fade its text and icon colors in the same system. Beverly's toast fade-out does both: it sets a mask opacity on each card's surface and lowers the alpha of its text and icon colors together.

## Demo

Run the interactive demo from the repository root:

```sh
cargo run --example clip_mask
cargo run --example clip_mask -- dark
```

The demo draws the same gradient square four times over a checkerboard, so transparency is visible:

- **No clip or mask:** the reference.
- **Clip:** its radius is controlled by the clip-radius slider (0 to 80 px).
- **Mask:** its radius and opacity are controlled by the mask-radius slider (0 to 80 px) and the mask-opacity slider (0 to 100 percent).
- **Clip + shadow:** a clipped square with a drop shadow, showing that the shadow is not trimmed.

The top-right toggle switches the page between light and dark mode.
