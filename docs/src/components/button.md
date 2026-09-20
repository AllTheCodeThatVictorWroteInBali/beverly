# Button

Buttons are the most common action primitive in Beverly. They should feel obvious, readable, and safe to activate. A button is not just a visual element; it is the clearest expression of a command in the interface, and it carries responsibility for clarity, hierarchy, and trust.

<img src="../assets/component-button.svg" alt="Button component illustration" width="860" />

## Overview

Use buttons for actions that change state, submit data, navigate to the next step, or trigger a workflow. They should be easy to scan, consistent across the app, and clearly distinct from content that only looks interactive.

This means a button should make its intent obvious even without examining the surrounding page. The label, role, and emphasis all contribute to that clarity.

## When to use

Use a button for a direct action such as submitting, navigating, confirming, or launching a workflow. Buttons should be used for commands, not for passive display or purely decorative emphasis.

If the control is not an action, a different primitive is usually more appropriate. The product feels better when button usage stays meaningfully narrow and intentional.

## Button styles

Beverly buttons are usually chosen from a small set of visual roles:

- primary for the main action in a surface
- secondary for supporting actions that are still important
- subtle for low-emphasis controls that should stay available without pulling focus
- accent for actions that benefit from stronger emphasis in a branded or highlighted context
- icon for compact toolbars or tight control groups where text would be redundant

Keep the hierarchy clear. Most surfaces should have one obvious primary action and a smaller number of supporting controls.

This prevents the page from collapsing into a visual pile of equally important buttons, which often makes the actual user task harder to infer.

## Basic example

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn spawn_primary_action(mut commands: Commands) {
    commands.spawn((
        NodeBundle::default(),
        BeverlyButton::primary("Deploy")
    ));
}
```

## Button variants

Different button styles should map to different levels of emphasis.

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn spawn_action_row(mut commands: Commands) {
    commands.spawn((
        NodeBundle::default(),
        BeverlyButtonGroup::new()
            .button("Preview")
            .button("Discard")
            .button("Publish")
    ));
}
```

In practice, a primary button should be reserved for the action the user is most likely to choose, while secondary and subtle buttons should support that choice without competing with it. Accent buttons are best used sparingly when the interface needs a stronger visual cue than the default hierarchy provides.

## Button tags and icon buttons

Buttons can appear with or without text, depending on the task.

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn spawn_toolbar(mut commands: Commands) {
    commands.spawn((
        NodeBundle::default(),
        BeverlyButtonGroup::new()
            .button("Save")
            .button("More")
    ));

    commands.spawn(BeverlyButton::icon("play"));
}
```

Use text buttons when the action name matters. Use icon buttons when space is constrained and the meaning is already obvious from the surrounding context.

## Sizing and width

Buttons should feel proportionate to the surrounding layout. Use a compact size for dense toolbars, a normal size for most content areas, and a larger presentation only when the action needs extra prominence.

Prefer full-width buttons only when the button is the dominant action in a narrow container, such as a mobile layout, a confirmation panel, or a short setup flow. Otherwise, let the button width follow its content so the layout stays easier to scan.

## States

A button should provide clear feedback for:

- default
- hover
- pressed
- focus-visible
- disabled
- loading

This helps users understand whether the action is ready, busy, or unavailable without guessing from layout or color alone.

This is especially important in systems with asynchronous operations, because a loading or disabled state should clearly communicate that the button cannot yet be activated again.

## Toggle behavior

Some button patterns behave like toggles or grouped choices rather than one-off actions. Use those patterns when the control needs to represent a persistent state or an exclusive selection.

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_toggle_row(mut commands: Commands) {
    commands.spawn(BeverlyButtonGroup::new()
        .button("List")
        .button("Grid")
        .button("Compact"));
}
```

Keep toggle-like buttons visually connected so the relationship is obvious, and make the current choice easy to identify from both sighted and keyboard navigation.

## Accessibility

- provide an accessible label
- keep keyboard activation working with Enter and Space
- avoid using color alone to convey state
- support reduced motion for animated press and hover transitions
- preserve a logical focus order inside button groups
- make disabled actions visibly unavailable and non-interactive

Buttons are one of the most common keyboard interaction points, so their accessibility is a high-value issue. A button that looks fine but cannot be reached or understood through keyboard navigation still fails its primary purpose.

## Implementation notes

Choose the least surprising control for the task. If the user needs a single definitive action, use one primary button. If they need to choose among a few nearby actions, use a button group. If the action is mostly navigational, consider whether a link or a different surface would communicate intent more clearly.

The button should make the product feel decisive instead of uncertain. That means the primary action should come first, secondary actions should remain visually subordinate, and low-priority controls should not distract from the task.

## Styling guidance

Use primary buttons for the main action, secondary buttons for supporting actions, and subtle buttons for low-priority choices. Keep spacing and touch targets consistent across screens so the interface feels predictable and easy to scan. When a page has many actions, reduce the number of high-emphasis buttons so the interface keeps a clear visual hierarchy.

## Summary

Beverly buttons should be clear, calm, and strongly aligned with the user’s task. They are one of the interface’s most important communication tools, so their hierarchy and behavior should be treated with the same care as the layout itself.
