# Accessibility Overview

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

Accessibility is part of the core design contract, not an afterthought. Beverly treats it as a first-class product requirement: every interactive surface should be understandable by keyboard, readable in high-contrast settings, resilient under reduced-motion preferences, and labeled clearly for assistive technologies.

## Core principles

- semantic structure before decoration
- visible focus and predictable focus order
- strong contrast without relying on color alone
- reduced-motion support for non-essential effects
- names, roles, and states that remain clear to screen readers

## The Beverly approach

The library should make it easy to build interfaces that are robust by default. That means using thoughtful component structure, consistent state handling, and accessible defaults instead of forcing each app to bolt accessibility on later.

A well-designed app keeps the same rules across surfaces:

- actions are reachable by keyboard
- focus is always visible and stable
- status changes are announced or obvious
- motion supports preference-based reduction
- text remains legible in light, dark, and high-contrast modes

## Recommended design flow

1. Start with clear hierarchy and semantic labels.
2. Add interactive behavior and keyboard flow.
3. Verify focus indicators and focus restoration.
4. Confirm contrast and motion preferences.
5. Check that critical state changes are understandable without visual flourish.

## Example: accessible app shell

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_shell(mut commands: Commands) {
    commands.spawn((
        NodeBundle::default(),
        BeverlyAppShell::new(),
    )).with_children(|parent| {
        parent.spawn(BeverlySidebar::new()).with_children(|sidebar| {
            sidebar.spawn(BeverlyNavItem::new("Overview").active(true));
            sidebar.spawn(BeverlyNavItem::new("Reports"));
            sidebar.spawn(BeverlyNavItem::new("Settings"));
        });

        parent.spawn(BeverlyMainPane::new()).with_children(|main| {
            main.spawn(BeverlyTitle::new("Workspace"));
            main.spawn(BeverlyText::new("A keyboard-friendly dashboard surface."));
            main.spawn(BeverlyButton::primary("Create report"));
        });
    });
}
```

This pattern keeps the hierarchy simple, the action labels explicit, and the focus path predictable.

Good accessibility is not a separate aesthetic. It is a baseline for clarity, confidence, and usability.
