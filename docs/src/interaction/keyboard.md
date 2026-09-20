# Keyboard

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

Keyboard support is a first-class requirement for Beverly interfaces. Dense, highly interactive applications are not complete unless their most important actions can be reached and operated without a mouse. Keyboard interaction is not a special mode; it is one of the primary ways users navigate, inspect, and operate a serious interface.

## Core expectations

- tab order matches the visible structure of the screen
- focus is always visible and stable
- action keys support the common interactive patterns users expect
- overlays trap focus appropriately and restore it when dismissed
- arrow-key and selection patterns should work for lists, menus, and grouped controls
- keyboard semantics should match the interface structure rather than rely on hidden behavior

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

This pattern keeps actions simple, visible, and keyboard-operable. The user can move through them in a predictable order without depending on pointer hover or hidden shortcuts.

## Designing for keyboard users

Keyboard interaction should feel natural and efficient. That means the interface should respect common conventions such as:

- Tab to move through interactive controls
- Shift+Tab to move backward
- Enter and Space to activate buttons or controls
- Escape to close overlays or dismiss transient states
- arrow keys to move through lists, menus, or grouped options

These expectations are not optional decorations. They are basic interaction patterns that users rely on for confidence and speed.

## Why keyboard support matters in Bevy UI

In a scene-driven, data-driven UI such as Bevy, it is easy to focus on what is displayed and forget that the interface must also maintain a navigable interaction model. The layout system may know where widgets are positioned, but the interaction layer must still define the actual path a user takes through them.

This makes keyboard support a structural concern, not just a final polish pass. Good keyboard flow depends on semantic structure, focus order, and stable state management.

## Guidance

Keyboard support should feel natural and predictable. A user should be able to navigate through panels, menus, and simple dialogs without learning custom hidden gestures or relying on hover states. Great keyboard support reduces friction because it makes the system feel controllable in a way that matches the user’s expectations.
