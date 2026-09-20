# Badge

Badges are compact status markers. They work well for states, counts, tags, or short labels.

## Example

```rust
fn build_status(mut commands: Commands) {
    commands.spawn(BeverlyBadge::new("Live")
        .variant(BadgeVariant::Success));
}
```

## Use examples

- online/offline states
- subscription tiers
- system health markers
- counts in lists or tables
