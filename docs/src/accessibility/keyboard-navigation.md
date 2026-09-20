# Keyboard Navigation

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

Keyboard navigation is a core interaction model, not a secondary mode. A design is not complete if a user cannot reach important actions, modify state, and move through content without a pointer.

## Required behavior

All major flows should support:

- Tab and Shift+Tab for sequential movement
- Enter and Space for activation
- arrow keys for list, menu, and selection patterns
- Escape to dismiss overlays or close transient controls
- clear focus order between panels, menus, and dialogs

## Layout and hierarchy

Keyboard order should follow the visual hierarchy and the semantic reading order. If a user can see a section before another, the focus order should usually follow the same flow. This reduces cognitive load and makes the interface feel stable.

When layouts become dense, such as tabs, tables, or toolbars, a component should define a predictable pattern rather than letting focus drift unpredictably based on DOM or render order.

## Controls with special behavior

Some widgets require more than a simple button pattern:

- menus should support arrow-key navigation and dismissal
- tabs should move focus between tabs without losing context
- table cells and rows should have a clear grid or list pattern
- modals should trap focus until dismissed
- search and filter controls should support immediate keyboard completion without pointer assistance

## Avoiding common failures

- do not rely on hover-only affordances
- do not hide interactive elements behind visual styling that only works with a mouse
- do not expose controls without labels
- do not create keyboard traps in nested panels or overlays

## Example: tab-like navigation

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_tabs(mut commands: Commands) {
    commands.spawn((
        NodeBundle::default(),
        BeverlyTabs::new(),
    )).with_children(|tabs| {
        tabs.spawn(BeverlyTab::new("Overview").active(true));
        tabs.spawn(BeverlyTab::new("Activity"));
        tabs.spawn(BeverlyTab::new("Settings"));
    });
}
```

A tab system should support arrow-key movement, visible selection state, and a predictable focus order so users can move between panels without a pointer.

## Good default rule

If a user can understand the interface, they should also be able to operate it with a keyboard. A Beverly UI should feel equally intentional in both pointer and non-pointer workflows.
