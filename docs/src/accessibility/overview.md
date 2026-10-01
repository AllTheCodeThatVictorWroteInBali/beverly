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

## Component checklist

Use the [component accessibility checklist](./component-checklist.md) during
implementation and review. It is the source of truth for the expected role,
name, state, keyboard, focus, disabled, validation, relationship, and
announcement behavior of each interactive component.

Every new component should add at least one headless semantic regression test,
one keyboard/focus test when it is composite, and a manual screen-reader check
for platform behavior that cannot be proven without a real accessibility
adapter.

## Semantic Relationships

`SemanticNode` carries a component's role, accessible name, value, state, and
actions. Its `SemanticRelationships` connects that node to other UI entities.
Beverly projects supported relationships into the AccessKit tree consumed by
the platform accessibility adapter:

| Relationship | Meaning |
| --- | --- |
| `labelled_by` | Another entity provides this node's name. |
| `described_by` | Other entities provide additional description. |
| `error_message` | An entity contains this control's error message. |
| `controls` | This node controls the referenced entities. |
| `owns` | This node owns the referenced entities in the semantic tree. |
| `active_descendant` | The referenced entity is the active item within this control. |

Relationship values are Bevy `Entity` IDs, not text or DOM IDs. The reverse
`controlled_by` field is retained in Beverly's semantic model, but AccessKit
has no corresponding property; set `controls` on the controlling node instead
so the relationship reaches assistive technology. Platform adapters and
screen readers can vary, so verify important flows with real assistive
technology as well as semantic projection tests.

For toggle buttons, set `SemanticState.pressed` only when reporting the
button's persistent on/off state. It is not the momentary pointer-pressed
interaction state. `checked`, when present, takes precedence because both map
to the same toggle state in AccessKit.

## Example: accessible app shell

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_shell(mut commands: Commands) {
    commands.spawn((
        NodeBundle::default(),
        BeverlyAppShell::new(),
    )).with_children(|parent| {
        parent.spawn(NodeBundle::default()).with_children(|sidebar| {
            sidebar.spawn(BeverlyButton::primary("Overview"));
            sidebar.spawn(BeverlyButton::secondary("Reports"));
            sidebar.spawn(BeverlyButton::secondary("Settings"));
        });

        parent.spawn(BeverlyMainPane::new()).with_children(|main| {
            main.spawn(BeverlyTitle::new("Workspace"));
            main.spawn(text("A keyboard-friendly dashboard surface."));
            main.spawn(BeverlyButton::primary("Create report"));
        });
    });
}
```

This pattern keeps the hierarchy simple, the action labels explicit, and the focus path predictable.

Good accessibility is not a separate aesthetic. It is a baseline for clarity, confidence, and usability.
