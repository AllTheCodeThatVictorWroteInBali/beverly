# Components Overview

Beverly components are the reusable building blocks that sit on top of Bevy's ECS and layout system. They are designed to feel native in Rust applications while still being themeable, accessible, and easy to compose into larger screens.

## Component model

A typical Beverly widget is a small stateful unit with these concerns:

- semantic structure and labels
- layout and sizing rules
- interaction state and keyboard focus
- theming via design tokens
- paint and material treatment when needed

This matches the pattern used in systems like block_studio: UI widgets are built from a small shared surface model, then specialized with different semantic and visual treatments. Title and text follow the same rule: they are lightweight content primitives that inherit the app's typography and surface rules instead of carrying their own isolated styling system.

## Title

Titles are the high-emphasis text primitive for screens, panels, cards, and section headers. Use them when you need a clear hierarchy marker that tells the user what area they are looking at.

In practice, titles work best when they are short, direct, and placed near the top of the relevant surface. The implementation pattern from block_studio is to treat title text as a reusable semantic block rather than a one-off label, so the same component can be used across app shells, cards, and page headers.

## Text

Text is the default body-content primitive. Use it for descriptions, helper copy, metadata, and any other supporting content that should inherit the app's global font and scale.

Text should stay flexible: the same primitive can render dense copy in a card, supporting notes under a title, or lightweight explanatory content in a panel. In block_studio-style composition, text is usually added as a child of a larger surface so it follows the same layout, spacing, and typography decisions as the rest of the interface.

## Composition pattern

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_dashboard(mut commands: Commands) {
    commands.spawn((
        NodeBundle::default(),
        BeverlyStack::vertical(),
    )).with_children(|parent| {
        parent.spawn(BeverlyCard::new("Summary")
            .with_body("Realtime metrics and AI actions"));
        parent.spawn(BeverlyButton::primary("Open report"));
    });
}
```

## Recommended usage

- Start with layout primitives such as stacks and containers.
- Add semantic content blocks such as cards and badges.
- Then layer in interactive controls such as buttons, toggles, and inputs.
- Finally apply rendering effects such as shadows, gradients, or glass surfaces only where they add clarity.

## Catalog

- buttons for primary actions
- title for section and page hierarchy
- text for supporting copy and body content
- inputs and selects for editable data
- modals and toasts for important feedback
- tables and pagination for dense data views
- navigation, tabs, and search for multi-surface applications

Each component should remain legible in light and dark themes and degrade gracefully when motion or contrast settings are reduced.
