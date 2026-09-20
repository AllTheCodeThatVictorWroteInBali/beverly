# Design Tokens

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

Design tokens define the naming and values used across Beverly interfaces. They provide a stable vocabulary for color, spacing, motion, radius, and other visual primitives so that the system can stay consistent as the product grows.

## Why tokens matter

A design system becomes difficult to maintain when the same values are redefined repeatedly in different screens and components. Without a shared token layer, a button might use one border radius, a card another, and a panel a different spacing scale. The result is a product that feels visually inconsistent even when each individual element is implemented perfectly.

Design tokens solve this by making the visual decisions explicit and reusable. The system does not depend on one-off values hidden in component implementations; instead, it relies on shared names that map to consistent values.

## Core uses of tokens

Design tokens are usually used for:

- color scales and semantic color roles
- spacing and layout rhythm
- border radius and surface shape
- typography sizes and line-height values
- motion durations and easing behavior
- shadow and elevation values

These values are then consumed by components and themes rather than being re-created at each use site.

## The value of semantic naming

The most useful tokens are not just raw numbers or strings. They are semantic decisions such as:

- surface.default
- accent.primary
- spacing.lg
- radius.md
- motion.fast
- shadow.panel

This makes the system easier to reason about because the design language maps to intent. A designer or developer can change the underlying value without disrupting the component relationships as long as the semantic role remains stable.

## Supporting theme switching

Design tokens are essential for theme support. Beverly can express multiple visual modes—light, dark, high-contrast, or branded variants—by swapping the token values while keeping the same semantic names.

This means the component logic stays stable while the visual language changes. A button still has the same role, but the theme decides whether that role appears as a light or dark treatment, a brighter accent, or a reduced-contrast fallback.

## Example token model

```rust
pub enum TokenRole {
    Background,
    Surface,
    Text,
    Accent,
    Success,
    Warning,
    Danger,
}

pub struct DesignTokenSet {
    pub color: ColorPalette,
    pub spacing: SpacingScale,
    pub radius: RadiusScale,
    pub motion: MotionScale,
}
```

This model makes it easier to reason about the theme as a cohesive system rather than a collection of unrelated visual tweaks.

## From tokens to components

A component should read from tokens, not hard-code values. This makes the app easier to evolve and safer to theme. For example, a button’s fill, text color, focus ring, and padding all derive from a small set of shared token decisions rather than being patched individually in each screen.

This has two major advantages:

1. visual consistency is maintained by default
2. the system can be themed without rewriting component logic

## Design token practice

A good token system stays small and intentional. It should clarify the design language without becoming an abstract catalog of every possible value. Beverly works best when the token set is a strong but manageable vocabulary: enough to express the product, but not so large that the system becomes hard to maintain.

## Summary

Design tokens are the connective tissue between product intent and implementation. They help Beverly keep its visual system coherent, themeable, and maintainable while preserving a clear separation between structure and style.
