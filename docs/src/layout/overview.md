# Layout Overview

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

Layout primitives define how content flows and aligns across screens. In Beverly, layout is not just a visual convenience; it is the structure that keeps interfaces readable, resilient, and easy to compose across different app shells and data densities.

## Core goals

- maintain a clear reading rhythm across panels and cards
- align content consistently without forcing all screens into the same rigid pattern
- support responsive composition for dashboards, forms, lists, and editor-style workspaces
- preserve hierarchy when content grows or shrink-wraps around dense controls

## Layout model

The layout system is designed around a few reusable patterns:

- containers for page and panel boundaries
- stacks for sequencing content vertically or horizontally
- grids for structured alignment and column-based layouts
- splits for multi-panel workspaces
- scroll regions for dense or overflow-heavy surfaces
- viewport-aware structures for full-screen or app-shell composition

## Example: dashboard shell

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_dashboard_shell(mut commands: Commands) {
    commands.spawn((
        NodeBundle::default(),
        BeverlyContainer::new(),
    )).with_children(|container| {
        container.spawn(BeverlyStack::vertical()).with_children(|stack| {
            stack.spawn(BeverlyTitle::new("Overview"));
            stack.spawn(BeverlyGrid::new().columns(3)).with_children(|grid| {
                grid.spawn(BeverlyCard::new("Revenue").with_body("$84.2k"));
                grid.spawn(BeverlyCard::new("Tasks").with_body("18 active"));
                grid.spawn(BeverlyCard::new("Alerts").with_body("3 pending"));
            });
        });
    });
}
```

## Design guidance

A strong layout system keeps visual rhythm predictable without making every screen feel templated. The right layout should make content easier to scan, not simply denser or more decorative.

Good layout decisions create a stable relationship between spacing, alignment, and priority. That makes it easier for users to understand what is important and how different sections relate to one another.
