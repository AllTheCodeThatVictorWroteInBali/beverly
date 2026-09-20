# Card

Cards group related content into a contained visual block. They are useful for dashboards, settings panels, and content summaries because they create a clear boundary without forcing a full-screen change of context.

## Example

```rust
use bevy::prelude::*;
use beverly::prelude::*;

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
- use consistent action placement when multiple cards share the same view

## Layout pattern

Cards work well in grids, stacks, or content columns. When cards represent a collection, align their headers and actions consistently so the relationship between similar items remains obvious.

## Accessibility

Keep card hierarchy readable and stable. Use headings, supporting copy, and action labels that make sense without relying on card color or decorative borders alone.
