# Themes

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

Themes let Beverly applications define a single visual language that can be swapped across light, dark, high-contrast, and branded variants without changing widget logic. A theme is the system’s visual contract: it says which tokens should be used for surfaces, text, accents, borders, states, and motion while preserving the same component architecture underneath.

A strong theme is not a style override. It is a stable layer that keeps the product coherent as it evolves.

## Why themes matter

A UI feels cohesive when the same design vocabulary is reused everywhere. If each widget chooses its own colors, spacing, and emphasis independently, the product becomes visually inconsistent and hard to maintain.

Beverly keeps these decisions at the theme layer so screens, panels, controls, and text all inherit the same design system. This keeps the product from drifting into a patchwork of ad hoc visual decisions.

## Built-in modes

Beverly is designed around a small set of standard theme modes:

- Light
- Dark
- High contrast
- Custom branded variants

These are not separate implementations of every component. Instead, they are alternate token sets that preserve the same component contracts while changing the visual result.

This ensures the product can move between a calm light theme, a dense dark theme, or a more contrast-focused mode without forcing the developer to rewrite the interface logic.

## Theme boundaries

A theme should define tokens and visual defaults, not application behavior. Components and policies decide how those values are applied in context.

That separation gives you a few useful properties:

- the app can switch between modes without rewriting screen logic
- semantically meaningful tokens remain stable across brand updates
- accessibility settings such as contrast and motion can adapt without breaking component structure
- the system can support product variants without duplicating every screen

This is a important separation between design and logic: the behavior of a button does not change because the palette changes, only its presentation.

## Example

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_app(mut commands: Commands) {
    commands.spawn((
        NodeBundle::default(),
        BeverlyAppShell::new(),
        BeverlyTheme::dark(),
    )).with_children(|parent| {
        parent.spawn(BeverlyCard::new("Deploy"));
        parent.spawn(BeverlyButton::new("Ship"));
        parent.spawn(BeverlyText::new("Ready to release"));
    });
}
```

The app shell owns the active theme, and child components read from it. That keeps cards, buttons, text, and form controls visually consistent even when the theme changes.

## Customization

Applications can override individual tokens or substitute an entirely different theme while keeping the same component interfaces. This makes it practical to support a branded variant or a system-specific palette without having to rework every screen.

In practice, a designer or product team can adjust the theme once and have the change propagate across the whole interface. This is one of the strongest reasons to keep theme logic centralized rather than spread across many individual widgets.

## Conceptual structure

This is the intended shape of the design system in simplified form:

```rust
let theme = Theme {
    colors: {
        base: "#0b1120",
        surface: "#111827",
        accent: "#7c3aed",
        success: "#22c55e",
        warning: "#f59e0b",
        danger: "#ef4444",
    },
    spacing: ["xs", "sm", "md", "lg", "xl"],
    radius: ["sm", "md", "lg"],
};
```

The important idea is that the application uses semantic tokens rather than scattered hard-coded values. The theme gives a product its visual identity; the components give it behavior and structure.

## Design-system value

Themes matter because they let an app carry a stable product identity while remaining flexible. They support accessibility, branding, and product variants without adding complexity to the screen architecture. In other words, the system can look different without requiring the underlying interaction model to change.

## Summary

Beverly’s theme model should be treated as foundational infrastructure: it is part of how the app remains consistent, accessible, and maintainable. A strong theme makes the interface feel intentional before the user even reads any content.
