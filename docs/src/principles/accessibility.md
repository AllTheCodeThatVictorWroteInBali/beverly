# Accessibility

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

Accessibility is a product feature and a design constraint. Beverly should treat it as a baseline quality indicator across every layer of the system, from component semantics to motion and contrast policy.

## The principle

A product is only as usable as its least accessible path. If a user cannot reach an action, understand a state, or recover from a modal, the system is not meeting its design contract.

## Practical rules

- accessible defaults should come before highly customized styling
- keyboard reachability should be treated as a requirement, not an optional mode
- contrast and focus should remain visible under theme changes
- reduced motion should preserve clarity rather than remove meaning
- labels and semantics should remain understandable even without visual emphasis

## Example: accessible primary action

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn render_accessible_action(mut commands: Commands) {
    commands.spawn(BeverlyButton::primary("Publish").with_label("Publish release"));
}
```

This principle is simple: every important action should be understandable, operable, and visually identifiable without requiring a pointer or subtle interpretation.

## Design expectation

A Beverly interface should feel confident in light mode, dark mode, high-contrast mode, and reduced-motion mode. If the app loses clarity under any of those conditions, it has not reached the standard the framework is intended to support.
