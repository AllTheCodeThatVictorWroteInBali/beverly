# Focus

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

Focus management ensures users can navigate interfaces predictably. It is the core mechanism that allows keyboard users to understand where they are and what they can interact with at any moment. In a dynamic UI, focus is the map that keeps the user oriented while the layout changes, modals open, and content is filtered or replaced.

## Why focus matters

Without a stable focus model, an interface becomes difficult to use even when the visual design is polished. Users often lose context when:

- a panel opens and focus is not moved into it
- a filtered list resets the selection unexpectedly
- a modal closes without returning the user to their original control
- a content update destroys the active element without warning

This creates a sense of disorientation and breaks the user’s workflow, especially for keyboard users and assistive technology users.

## Recommended rules

- focus should be visible on every interactive element
- dialog and overlay focus should move into the relevant container
- returning from a modal or panel should restore the triggering context
- dynamic content updates should not cause focus to vanish unexpectedly
- focus order should reflect the logical structure of the interface
- focus should be recoverable after state refreshes and filtered content changes

## Example

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn open_panel(mut commands: Commands, mut focus: FocusState) {
    let previous = focus.current();

    commands.spawn(BeverlyDrawer::new("Preferences")
        .on_close(move |world| {
            if let Some(el) = previous {
                world.focus(el);
            }
        }));
}
```

This pattern keeps the user in context. The focus does not disappear into a black box; it is intentionally restored to the element that triggered the panel.

## Focus in dynamic interfaces

A real application is always changing. Panels open, lists filter, data refreshes, and overlays appear. Good focus management does not assume the interface is static. Instead, it keeps the active element meaningful as the system updates around it.

This is especially important in Bevy-like ECS systems where entities are created and destroyed as part of normal UI lifecycle behavior. Focus transitions should be predictable, not accidental.

## Guidance

Focus is not just a visual ring. It is the path that keeps the user oriented in a system of changing panels, filtered lists, and modal interactions. A focus-aware interface feels stable and trustworthy because the user always knows where they are and what can be acted on next.
