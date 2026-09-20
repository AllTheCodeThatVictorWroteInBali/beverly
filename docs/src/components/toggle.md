# Toggle

Toggles are compact on/off controls meant for settings and feature switches.

## Example

```rust
fn build_mode_toggle(mut commands: Commands) {
    commands.spawn(BeverlyToggle::new("Compact mode")
        .value(true));
}
```

## Interaction notes

Toggles should update immediately but still be understandable. Include a label and ensure the change is reflected in the state of any related settings or data views.
