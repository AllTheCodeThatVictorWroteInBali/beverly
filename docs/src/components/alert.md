# Alert

Alerts communicate notable information, warnings, or errors without blocking the entire screen.

## Example

```rust
fn build_alert(mut commands: Commands) {
    commands.spawn(BeverlyAlert::new("Storage nearly full")
        .variant(AlertVariant::Warning)
        .action("Review"));
}
```

## Guidance

Alerts should be brief, actionable, and easy to dismiss when appropriate.
