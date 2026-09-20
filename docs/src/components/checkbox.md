# Checkbox

Checkboxes represent independent on/off choices within a form or settings surface.

## Example

```rust
fn build_preferences(mut commands: Commands) {
    commands.spawn(BeverlyCheckbox::new("Enable notifications")
        .checked(true));
}
```

## Behavior

- independent selection state
- can be grouped with other checkboxes
- should expose a visible label and optional description
- should participate in keyboard navigation and form submission state

## When not to use

Use a radio group when the user must choose exactly one option from a set. Use a checkbox when multiple independent selections are valid.
