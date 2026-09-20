# Shaders

Shaders are the most powerful way to express the "look" of a Beverly surface without creating one-off widget code. In theblocks_studio-style UI stacks, shader work sits behind a shared material interface so widgets remain consistent while their surface properties change.

## Pattern

```rust
// Conceptual shader use
let material = UiShapeMaterial {
    fill_color: Color::WHITE,
    border_color: Color::srgba(0.7, 0.7, 0.9, 1.0),
    corner_radius: 20.0,
    ..default()
};
```

## Why use shaders here?

- rounded surfaces and soft edges
- glass, blur, and refraction effects
- accent glow and focus treatment
- thematic gradients without brittle per-widget code

## Guidance

Keep shader logic predictable and bounded. Prefer a single material model with runtime parameters over many specialized shader variants. This keeps rendering costs easier to reason about and helps preserve accessibility policies.
