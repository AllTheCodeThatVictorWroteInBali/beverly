# Materials

Materials connect a UI surface to its paint behavior. A material is the place where the app defines how a widget is filled, how it reacts to focus, and how it handles effects like blur, highlight, or noise.

## Example

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn spawn_material_surface(mut commands: Commands) {
    commands.spawn((
        NodeBundle::default(),
        BeverlySurface::new()
            .with_fill(Color::srgb(0.08, 0.09, 0.15))
            .with_border(Color::srgb(0.55, 0.68, 1.0), 1.5)
            .with_material(MaterialPreset::Elevated),
    ));
}
```

## Material goals

- maintain a semantic surface layer instead of ad hoc color values
- keep the same material contract across components
- allow theme swaps without rewriting widgets
- make effect tuning easy to adjust in one place
