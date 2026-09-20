# Rendering Overview

Beverly rendering is intentionally material-first: surfaces, paint layers, and shader-backed effects are treated as part of the UI system rather than as app-specific hacks. The design borrows from the practical reference stack used in theblocks_studio, where a shared `Surface` model is extended with gradients, shadows, blur, and glass-like material treatments.

## Core ideas

- UI surfaces carry semantic paint and border information
- material parameters are data-driven, not hardcoded per widget
- shader effects are used only when they improve readability or depth
- reduced-motion and reduced-transparency safeguards remain part of the design contract

## Typical rendering pipeline

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_glass_panel(mut commands: Commands) {
    commands.spawn((
        NodeBundle::default(),
        BeverlySurface::new()
            .with_material(MaterialPreset::LiquidGlass)
            .with_corner_radius(24.0),
    ));
}
```

## Effects stack

The rendering stack usually progresses from:

1. base surface fill
2. gradient and border painting
3. shadow and highlight layers
4. blur or translucency
5. focus and interaction overlays

This keeps the UI legible while allowing richer branded surfaces.
