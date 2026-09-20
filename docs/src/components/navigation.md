# Navigation

Navigation components help users move between major parts of an app or workspace.

## Example

```rust
fn build_nav(mut commands: Commands) {
    commands.spawn(BeverlyNavigation::new()
        .link("Home")
        .link("Library")
        .link("Reports")
        .link("Settings"));
}
```

## Patterns

- side navigation for dense app shells
- top navigation for lighter content layouts
- breadcrumb trails for hierarchical drill-downs

Navigation should be stable, scannable, and reflect the current page or context clearly.
