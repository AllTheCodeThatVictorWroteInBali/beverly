# Tooltip

Tooltips provide contextual help for controls or labels without permanently consuming layout space.

## Example

```rust
fn build_tooltip(mut commands: Commands) {
    commands.spawn(BeverlyTooltip::new("Sync now")
        .text("Starts a full metadata refresh"));
}
```

## Use it when

- a control needs a quick explanation
- visual affordance is not obvious
- the extra text will distract if always visible
