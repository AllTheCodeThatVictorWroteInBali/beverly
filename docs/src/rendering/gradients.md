# Gradients

Gradients add polish and visual hierarchy to surfaces without requiring custom widget logic. They are often used for hero panels, accent areas, and interactive emphasis.

## Example

```rust
fn build_gradient_panel(mut commands: Commands) {
    commands.spawn((
        NodeBundle::default(),
        BeverlySurface::new()
            .with_linear_gradient((0.0, 0.0), (1.0, 1.0), [
                Color::srgb(0.15, 0.48, 1.0),
                Color::srgb(0.43, 0.25, 0.9),
            ]),
    ));
}
```

## Guidance

- use gradients to direct attention, not as a replacement for strong hierarchy
- keep contrast high enough for readable text
- prefer soft, controlled transitions over dramatic color shifts
