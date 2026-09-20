# Toggle

Toggles are compact on/off controls meant for settings and feature switches. They are ideal when the user needs a binary choice that should be easy to scan and quick to change.

## Example

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_mode_toggle(mut commands: Commands) {
    commands.spawn(BeverlyToggle::new("Compact mode")
        .value(true));
}
```

## Interaction notes

Toggles should update immediately but still be understandable. Include a label and ensure the change is reflected in the state of any related settings or data views.

## Accessibility

Toggle controls need a clear label, visible state, and keyboard support. The selected state should be understandable even for users who are not relying on the glyph or color treatment alone.
