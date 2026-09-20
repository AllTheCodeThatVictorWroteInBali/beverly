# Button Group

Button groups cluster related actions that belong together while maintaining a shared visual rhythm.

## Example

```rust
fn build_actions(mut commands: Commands) {
    commands.spawn(BeverlyButtonGroup::new()
        .button("Preview")
        .button("Save")
        .button("Publish"));
}
```

## Usage guidance

Group related controls when they share a common purpose. Avoid mixing emergency actions and low-priority actions in the same cluster without clear visual hierarchy.
