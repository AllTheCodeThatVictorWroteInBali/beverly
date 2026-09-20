# Gradients

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

Gradients add polish and visual hierarchy to surfaces without requiring custom widget logic. They are often used for hero panels, accent areas, and interactive emphasis. In a design system, gradients are best treated as a controlled surface tool rather than as a free-form visual flourish layered onto every card.

## What gradients do well

A gradient can help the interface do several useful things:

- establish brand tone in a hero or shell surface
- create subtle emphasis without relying on a harsh solid fill
- signal focus or strong state without creating an overly loud border
- add a premium feel to overlays, panels, or onboarding surfaces

They are especially effective when used in limited, purposeful places. A gradient can help define a visual hierarchy, but it should not replace structure, spacing, or typography as the source of meaning.

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

This is a good example of a gradient used as a controlled surface treatment. It adds visual richness while leaving the content hierarchy intact and the interaction model clear.

## Guidance

- use gradients to direct attention, not as a replacement for strong hierarchy
- keep contrast high enough for readable text
- prefer soft, controlled transitions over dramatic color shifts
- keep brand gradients limited to key surfaces rather than spreading them through the entire app
- ensure gradients remain readable under dark, light, and high-contrast modes

## Gradients and content legibility

A gradient is only successful when the surrounding content remains easy to read. A bright, saturated gradient behind dense text can quickly reduce legibility and make the interface harder to use.

This is why Beverly should favor gradients that are:

- soft rather than aggressive
- harmonious with the app’s color system
- easy to pair with text and border contrast
- intentional and limited in scope

## Theme-aware use

Gradients should feel like part of the theme, not as random decoration. The system should use tokens and colors from the product design language so gradients remain consistent across surfaces and screens.

A gradient used in one screen should feel like it belongs to the same visual system as the rest of the product, not like a special effect imported from elsewhere.

## The core idea

Gradients are a surface-level tool for emphasis and atmosphere. They should add quality to the design without introducing visual noise or reducing readability. In a strong design system, they are used purposefully, consistently, and with accessibility in mind.
