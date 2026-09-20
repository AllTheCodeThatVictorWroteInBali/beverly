# Shadows

Shadows communicate depth and focus without overpowering interface content. Beverly uses a small set of drop-shadow presets so surfaces feel elevated consistently instead of feeling arbitrarily heavy or soft.

## Drop shadows

Use drop shadows to separate a surface from the background when it should feel like it sits on top of the page. They are especially useful for cards, floating toolbars, popovers, and other elements that need a clear sense of elevation.

### Small

Use the small shadow for subtle lift. This is the right choice for default cards, compact buttons, hover feedback, and other places where the interface should feel light and refined.

Small shadows should be gentle enough that they do not compete with the content itself. The effect is mostly about orienting the eye, not announcing the surface.

### Regular

Use the regular shadow for the default elevated surface. This is the most common shadow level for panels, card groups, inline popovers, and other important but not dramatic surfaces.

It gives enough separation to read as a distinct layer without becoming obvious or decorative. In most layouts, this is the standard choice.

### Large

Use the large shadow for major floating surfaces such as modals, drawers, high-priority panels, and other UI that needs strong separation from the main page. This shadow is more visible because the surface is farther from the background and needs more emphasis.

Large shadows should be used selectively. If every surface uses a large shadow, the interface loses hierarchy and starts to feel noisy.

## Rule of thumb

- use the smallest shadow that carries the needed elevation
- keep all shadows in the same system so the app feels coherent
- reserve the largest shadow for decisive, high-priority overlays
- avoid stacking multiple heavy shadows on a single element

## Example

```rust
commands.spawn(BeverlyCard::new("Overview")
    .with_shadow(ShadowPreset::Small));

commands.spawn(BeverlyCard::new("Metrics")
    .with_shadow(ShadowPreset::Medium));

commands.spawn(BeverlyModal::new("Settings")
    .with_shadow(ShadowPreset::Large));
```

The small, regular, and large presets are not arbitrary sizes; they define a shared vocabulary for elevation across the interface.
