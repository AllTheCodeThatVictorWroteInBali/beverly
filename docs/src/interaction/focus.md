# Focus

Focus management ensures users can navigate interfaces predictably. It is the core mechanism that allows keyboard users to understand where they are and what they can interact with at any moment.

## Recommended rules

- focus should be visible on every interactive element
- dialog and overlay focus should move into the relevant container
- returning from a modal or panel should restore the triggering context
- dynamic content updates should not cause focus to vanish unexpectedly

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

## Guidance

Focus is not just a visual ring. It is the path that keeps the user oriented in a system of changing panels, filtered lists, and modal interactions.
