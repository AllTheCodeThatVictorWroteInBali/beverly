# Focus Management

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

Focus management keeps users oriented when an interface changes, opens panels, or updates content dynamically. In practice, a good focus model is predictable, visible, and easy to recover from when the interface state changes.

## Design rules

- Every interactive control should have a clear focus state.
- Focus should move in a logical order that matches the visual and reading order.
- When a modal appears, focus should move into it and return on close.
- When content is inserted or replaced, focus should land on the most relevant next element.
- Focus should never disappear because a widget re-rendered.

## Recommended behavior

A Beverly pattern should keep the active element stable while still allowing the app to respond to user intent. For example, when a dialog opens, the first actionable element should receive focus. When the dialog closes, focus should return to the triggering control unless the interface clearly indicates a different destination.

Dynamic panels, drawers, tabs, and inline editors should preserve context. If the user is deep in a workflow, an app should not reset them to the top of the page or trap them in a visually hidden state.

## Visual focus

The focus indicator should be strong enough to stand out without being overly decorative. In dark and light themes, it should remain visible across surfaces, not disappear on glass panels or translucent containers.

A focus treatment should be:

- high enough contrast to be unmistakable
- consistent across controls
- applied to keyboard focus, not just pointer hover
- easy to read against gradients, shadows, and atypical backgrounds

## Implementation guidance

Focus should be managed through shared behavior rather than ad hoc fixes inside individual widgets. The application shell, modal layer, and navigation container should all cooperate around the same focus policy.

A strong default pattern is:

- define a logical tab order for each shell
- provide a visible focus ring on interactive elements
- restore focus after dismissing overlays
- avoid auto-focus in situations that interrupt the user without clear context

## Example: focus restoration after a modal

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn open_dialog(mut commands: Commands, mut focus: FocusState) {
    let trigger = focus.current();

    commands.spawn(BeverlyModal::new("Delete item")
        .body("This action cannot be undone.")
        .confirm_label("Delete")
        .cancel_label("Cancel")
        .on_close(move |world| {
            if let Some(element) = trigger {
                world.focus(element);
            }
        }));
}
```

This pattern keeps the user anchored in their task: when the dialog ends, focus returns to the control that opened it instead of jumping unpredictably elsewhere.

Focus is not just a visual effect; it is the navigation system for keyboard users.
