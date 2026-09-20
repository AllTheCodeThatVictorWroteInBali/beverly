# Blur

Blur is one of the most effective ways to show depth and separation in a layered interface. Used sparingly, it helps create readable glass or ambient surfaces without reducing legibility.

## Example

```rust
fn spawn_blurred_backdrop(mut commands: Commands) {
    commands.spawn((
        NodeBundle::default(),
        BeverlySurface::new()
            .with_blur(18.0)
            .with_tint(Color::srgba(0.1, 0.12, 0.18, 0.55)),
    ));
}
```

## Use cases

- modal overlays
- floating panes and sidebars
- translucent chrome and utility surfaces
- layered data views with a quiet background

## Accessibility note

If blur reduces contrast too much, the surface should fall back to a more opaque treatment or a higher-contrast border.
