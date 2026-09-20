# Viewport

Viewport primitives support full-screen composition and responsive surfaces. They are the right abstraction when the app needs a full-bleed shell, immersive canvas, or global layout container that reacts to the available screen space.

## When to use

Use a viewport for:

- full-screen editor surfaces
- media or playback experiences
- app-shell composition with persistent chrome
- screens that should adapt directly to browser or window size

## Example

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_viewport(mut commands: Commands) {
    commands.spawn((
        NodeBundle::default(),
        BeverlyViewport::new(),
    )).with_children(|viewport| {
        viewport.spawn(BeverlyAppShell::new());
    });
}
```

## Guidance

Viewport-based composition is powerful, but it should preserve a clear hierarchy and safe margins. The user should feel grounded in the interface even when the surface is immersive or full-bleed.
