# Tabs

Tabs organize several related views within the same area. They are useful for settings, dashboards, or content sections that share a common context.

## Example

```rust
fn build_tabs(mut commands: Commands) {
    commands.spawn(BeverlyTabs::new()
        .tab("Overview")
        .tab("Activity")
        .tab("Settings"));
}
```

## Guidance

- use tabs for local navigation, not global app navigation
- keep the number of tabs manageable
- ensure the active tab is visually and semantically clear
- support arrow-key navigation when possible
