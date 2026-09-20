# Button

Buttons are the most common action primitive in Beverly. They should feel obvious, readable, and safe to activate.

## When to use

Use a button for a direct action such as submitting, navigating, confirming, or launching a workflow.

## Basic example

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn spawn_primary_action(mut commands: Commands) {
    commands.spawn((
        NodeBundle::default(),
        BeverlyButton::new("Deploy")
            .variant(ButtonVariant::Primary)
            .icon("rocket")
    ));
}
```

## States

A button should provide clear feedback for:

- default
- hover
- pressed
- focus-visible
- disabled
- loading

## Accessibility

- provide an accessible label
- keep keyboard activation working with Enter and Space
- avoid using color alone to convey state
- support reduced motion for animated press and hover transitions

## Styling guidance

Use primary buttons for the main action, secondary buttons for supporting actions, and subtle buttons for low-priority choices. Keep spacing and touch targets consistent across screens.
