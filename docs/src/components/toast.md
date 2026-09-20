# Toast

Toasts display brief status changes such as successful saves, queued jobs, or transient warnings.

## Example

```rust
fn show_success_toast(mut commands: Commands) {
    commands.spawn(BeverlyToast::success("Saved successfully"));
}
```

## Best practices

- keep messages short and informational
- avoid stacking too many toasts at once
- support dismissal and a clear lifetime
- prefer live status updates in the UI when the state must remain visible
