# Shadows

Shadows help show elevation and surface separation. They are best when subtle and consistent, and they should support the interface without becoming decoration.

## Drop shadow sizes

Beverly keeps a small set of shadow presets so surfaces can communicate depth in a predictable way.

### Small

Use a small shadow for light lift: compact cards, hover states, quick actions, and other subtle surfaces that should feel slightly separated from the background.

### Regular

Use the regular shadow for the default elevated surface. This is the standard choice for cards, property panels, and other content that needs a clear separation but should still feel understated.

### Large

Use the large shadow for major floating overlays: modals, drawers, menus, and panels that need a strong sense of separation from the page beneath them.

## Example

```rust
fn elevated_card(mut commands: Commands) {
    commands.spawn(BeverlyCard::new("Overview")
        .with_shadow(ShadowPreset::Small));

    commands.spawn(BeverlyCard::new("Metrics")
        .with_shadow(ShadowPreset::Medium));

    commands.spawn(BeverlyModal::new("Settings")
        .with_shadow(ShadowPreset::Large));
}
```

## Common uses

- cards and panels
- modal shells and overlays
- hover states on floating controls
- tooltips and context menus

## Principle

Use a single shadow system with a small set of presets so the app stays coherent across components. Pick the lightest shadow that still communicates the right depth.
