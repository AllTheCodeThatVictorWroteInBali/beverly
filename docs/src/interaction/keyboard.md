# Keyboard

Keyboard support is a first-class requirement for Beverly interfaces. Dense, highly interactive applications are not complete unless their most important actions can be reached and operated without a mouse.

## Core expectations

- tab order matches the visible structure of the screen
- focus is always visible and stable
- action keys support the common interactive patterns users expect
- overlays trap focus appropriately and restore it when dismissed

## Example

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_keyboard_action_row(mut commands: Commands) {
    commands.spawn(BeverlyToolbar::new()).with_children(|toolbar| {
        toolbar.spawn(BeverlyButton::primary("Save"));
        toolbar.spawn(BeverlyButton::secondary("Cancel"));
        toolbar.spawn(BeverlyButton::subtle("More"));
    });
}
```

## Guidance

Keyboard support should feel natural and predictable. A user should be able to navigate through panels, menus, and simple dialogs without learning custom hidden gestures or relying on hover states.
