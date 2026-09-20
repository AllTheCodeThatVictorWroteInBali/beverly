# Grid

Grid layouts support structured alignment across rows and columns. They are ideal for dashboards, forms with repeated fields, data cards, and dense surfaces where alignment matters more than free-form stacking.

## When to use

Use a grid when the interface contains multiple comparable items that should align cleanly. Examples include metric cards, list summaries, resource maps, and multi-column settings layouts.

## Example

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_grid(mut commands: Commands) {
    commands.spawn((
        NodeBundle::default(),
        BeverlyGrid::new().columns(3),
    )).with_children(|grid| {
        grid.spawn(BeverlyCard::new("Revenue").with_body("$84.2k"));
        grid.spawn(BeverlyCard::new("Tasks").with_body("18 active"));
        grid.spawn(BeverlyCard::new("Alerts").with_body("3 pending"));
    });
}
```

## Guidance

Grid layouts work best when the content is visually comparable. Keep column counts stable and avoid overcomplicating the rhythm with unusual spacing or inconsistent card sizes.
