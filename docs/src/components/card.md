# Card

Cards group related content into a contained visual block. They are useful for dashboards, settings panels, and content summaries.

## Example

```rust
fn build_summary_card(mut commands: Commands) {
    commands.spawn(BeverlyCard::new("Team health")
        .with_body("5 alerts resolved this week")
        .with_footer("Updated 3 minutes ago"));
}
```

## Best practices

- use cards to group related information
- keep headers short and specific
- pair dense cards with clear spacing and readable typography
- avoid stacking too many cards in a single viewport without a clear scan path

## Layout pattern

Cards work well in grids, stacks, or content columns. When cards represent a collection, align their headers and actions consistently.
