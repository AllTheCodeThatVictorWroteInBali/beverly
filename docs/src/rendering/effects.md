# Effects

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

Beverly effects should emphasize state and hierarchy instead of distracting from the content. Effects are most useful for focus, depth, and transitions between states. In a well-designed interface, effects should support meaning: they help the user understand that something is selected, active, elevated, or important, without becoming decoration for its own sake.

## What counts as an effect

Effects are the visual treatments layered on top of a surface after the base structure is established. They include things such as:

- shadows
- glows and accents
- hover and press changes
- soft blur behind overlays
- translucent fills and overlays
- focus rings or emphasis outlines

These treatments should enhance clarity, not replace it. If a control becomes difficult to understand because the effect is too strong or too noisy, the design has crossed the line from helpful to distracting.

## Example

```rust
fn build_effect_card(mut commands: Commands) {
    commands.spawn((
        NodeBundle::default(),
        BeverlyCard::new("Sync status")
            .with_shadow(ShadowPreset::Soft)
            .with_glow(Color::srgb(0.35, 0.75, 1.0)),
    ));
}
```

This pattern is intentionally restrained. The glow and shadow raise the visual priority of the card without needing a giant custom implementation. The card remains a card; the effect communicates emphasis and separation.

## Good effect patterns

- subtle hover lift
- focus rings on interactive controls
- soft elevation for modal and overlay surfaces
- restrained transparency for high-value UI containers
- subtle directional light or highlight response on premium surfaces

Effects are most useful when they support semantics. A focus ring should help the user locate the active control. A shadow should clarify depth. A glow should suggest emphasis without taking over the whole surface.

## How effects fit the system

A well-structured UI framework treats effects as layer-level behavior rather than ad hoc widget styling. This keeps them aligned with the architecture. Layout determines shape and position. Materials define the base look. Effects add emphasis and depth. Policies decide whether those effects are appropriate in a given context.

This clean separation matters because it prevents each widget from absorbing its own visual logic. Instead, effect use remains consistent and easier to adjust when the design changes.

## Reducing visual noise

As in theblocks_studio, effects should be controlled by a shared policy layer. This makes it easy to disable them for reduced motion, reduced transparency, or high-contrast applications.

A consistent policy layer is important because the same effect can be appropriate in one context and excessive in another. For example:

- a subtle shadow may be perfect for a card in normal mode
- the same shadow may be too heavy in a high-contrast mode
- a glow may be charming in a hero panel but distracting in a dense dashboard

A design system should not rely on every widget to decide these trade-offs independently.

## Effects should support hierarchy

Effects are most useful when they guide attention without crowding content. The strongest use case is hierarchy: helping the user see what is active, lifted, selected, or important.

This is why Beverly’s effect model should be constrained, not maximal. Effects are a tool for emphasis, not a substitute for strong structure, spacing, and typography.

## The right level of intensity

The best effect system is measured and compact. A small set of effect presets or parameters is easier to reason about than a large matrix of bespoke stylings. That leads to:

- more visual consistency
- less design drift between screens
- easier theming and product evolution
- better accessibility behavior under reduced transparency and motion settings

## The key idea

Effects should help users understand the interface, not compete with it. In a structured design system, they are one layer of emphasis operating within a stronger visual hierarchy. That keeps the product polished without making it noisy or hard to use.
