# Shadows

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

Shadows help show elevation and surface separation. They are best when subtle and consistent, and they should support the interface without becoming decoration. In a well-designed system, shadows are not a visual gimmick; they are a way to communicate depth, layering, and separation in a controlled, readable way.

## What shadows communicate

A shadow is one of the clearest ways to tell a user that one surface sits above another. It helps the eye understand:

- which panel is floating
- which control is active or elevated
- which content is layered above the background
- which overlay needs stronger separation from the page beneath it

This makes shadows a useful tool for hierarchy, especially in dense or highly structured interfaces.

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

This is an ideal use of a small shadow system: each surface gets the amount of elevation it needs without a large amount of custom styling.

## Common uses

- cards and panels
- modal shells and overlays
- hover states on floating controls
- tooltips and context menus
- data surfaces that need a clear sense of separation from the background

## Good shadow behavior

A strong shadow system should be:

- consistent across the app
- tuned to the actual elevation scale
- subtle enough to preserve readability
- distinct enough to communicate hierarchy
- easy to adjust for dark and light themes

The goal is not to create heavy, dramatic shadows everywhere. It is to establish a clear system of depth that feels calm and intentional.

## Avoiding over-shadowing

Too many strong shadows make the interface feel noisy and visually crowded. If every card and panel has a large visible shadow, the hierarchy becomes muddy and the app loses calmness.

This is why Beverly should favor a small set of preset values rather than a free-form shadow system that encourages each component to tune itself independently. A restrained preset model makes the product feel coherent.

## Shadows and accessibility

Shadows are not a replacement for contrast or focus states. They should support a surface, not hide a problem. In a high-contrast or reduced-visual-noise environment, a strong shadow may become too heavy or unnecessary.

That means shadow intensity should be part of the design policy layer, not just a static decoration value. The system should be able to reduce or suppress certain effects when accessibility or theme constraints require it.

## The principle

Use a single shadow system with a small set of presets so the app stays coherent across components. Pick the lightest shadow that still communicates the right depth. The best shadows are the ones users do not consciously notice because they make the structure clearer without interrupting the content.
