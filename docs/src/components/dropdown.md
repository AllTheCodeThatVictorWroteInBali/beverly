# Dropdown

Dropdowns reveal a contextual menu of actions or choices without permanently taking up screen space.

## Example

```rust
fn build_menu(mut commands: Commands) {
    commands.spawn(BeverlyDropdown::new("Actions")
        .item("Rename")
        .item("Duplicate")
        .item("Archive"));
}
```

## Best practices

- keep action menus focused and short
- ensure destructive actions are explicit
- close the menu when the user chooses an item or clicks outside the control
- support keyboard navigation and focus return
