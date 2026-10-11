# State Decorations

A `Surface` can carry semantic decorations separately from its fill and border. Focus rings are rendered by Beverly's surface shader; selection and validation currently reserve API slots but do not produce visuals.

## Authored Focus Rings

Use an authored ring when a surface needs a fixed focus treatment or when building a custom control. A ring is part of `Surface::decorations`, independent from the ordinary surface border:

```rust
use beverly::rendering::{FocusRing, FocusRingLayer, FocusRingPlacement, Paint, Surface};
use bevy::prelude::*;

let surface = Surface::rounded_rect_fill(8.0, Paint::solid(Color::WHITE))
    .with_focus_ring(
        FocusRing::outside(3.0, 3.0, Paint::solid(Color::srgb(0.12, 0.42, 0.92)))
            .with_placement(FocusRingPlacement::Outside)
            .with_secondary(
                FocusRingLayer::new(1.0, 0.0, Paint::solid(Color::WHITE))
                    .with_opacity(0.95),
            ),
    );
```

`FocusRing::outside(width, offset, paint)` creates the primary layer. Width and offset are in logical pixels. Placement determines how the ring relates to the surface boundary:

| Placement | Result |
| --- | --- |
| `Outside` | The ring sits beyond the surface edge. |
| `Center` | The ring straddles the surface edge. |
| `Inside` | The ring sits inside the surface edge. |

A `FocusRing` has one primary layer, an optional secondary layer, and an optional `OuterGlow`. Each `FocusRingLayer` has its own width, offset, opacity, and `Paint`. Use `.with_secondary(...)` and `.with_glow(...)` to add those optional treatments. The shader sanitizes negative or non-finite dimensions and clamps opacity to 0–1; ring width and offset are also capped to keep rendering bounds finite.

Rings and glows do not reserve layout space. Leave room outside the surface for an outside ring or glow; ancestors that clip their children may cut off that space. On the surface itself, `Clip` trims the focus ring, while `Mask` does not. See [Clipping and Masking](clipping-masking.md).

To change or remove an authored ring at runtime, update the surface's decorations or clear it:

```rust
surface.decorations.focus_ring = Some(ring);
*surface = surface.clone().clear_focus_ring();
```

## Automatic Focus Rings

`BeverlyPlugin` installs the accessibility focus-ring system. When focus moves to an entity with a `Surface`, Beverly writes a theme-derived ring into that surface's `decorations.focus_ring`; when focus leaves it, Beverly clears the previous automatic ring. This is separate from the surface border. On a focused surface, the automatic ring replaces any authored ring in that slot.

The default theme uses the focus color for `FocusIntent::Default`, the info color for `Selected`, and the error color for `Invalid`. The automatic focus system currently requests the default intent. `ThemeColors::focus_ring` also adapts the ring to its context and accessibility visual policy:

- Normal contrast uses a 2-pixel primary ring with a 2-pixel offset. It may add a contrasting secondary ring for high-contrast mode or for Glass, Accent, and Danger surface tones.
- High-contrast mode uses a 3-pixel primary ring and chooses black or white based on the background. It also adds a contrasting secondary layer and suppresses the glow.
- In normal contrast, a theme-colored outer glow is included unless reduced effects are enabled.

Focus visibility is controlled by `FocusVisibilityPolicy`. Its default is `KeyboardOnly`, so the ring appears when focus is visible from keyboard navigation. `Always` and `Programmatic` show the ring for any focused entity; `Hidden` suppresses it. The policy is initialized from `UI_FOCUS_VISIBILITY=always`, `programmatic`, or `hidden`; unset or unrecognized values use `KeyboardOnly`. High contrast and reduced effects can be requested at startup with `UI_HIGH_CONTRAST=1` and `UI_REDUCED_EFFECTS=1`.

Keyboard navigation requires a `TabGroup` and focusable elements with `TabIndex`; Beverly's built-in Tabs component handles roving focus within its tab list. See [Keyboard Navigation](../accessibility/keyboard-navigation.md). The demo combines the automatic ring on built-in tabs with authored ring controls and placement comparisons.

## Reserved Decoration Slots

`Decorations` also provides `selection` and `validation` fields, with `SelectionDecoration` and `ValidationDecoration` types. They are reserved semantic slots and are not rendered yet. Setting either value does not draw a selection outline, validation border, or other indicator. Until those renderers exist, represent selected or invalid state with an explicit surface fill, border, text, or an authored ring.

## Demo

Run the interactive example from the repository root:

```sh
cargo run --example state_decorations
cargo run --example state_decorations -- dark
```

Use Tab to enter the tabs, then the arrow keys to switch the active tab. The authored-ring controls change placement, intent color, width, offset, secondary layer, and glow. The three comparison swatches keep their Outside, Center, and Inside placements while following the selected width and color.
