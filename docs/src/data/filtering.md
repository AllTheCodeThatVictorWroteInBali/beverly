# Filtering

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

Filtering reduces noise and helps users focus on relevant information. A useful filter system makes the user feel in control without hiding the context of the underlying data set.

## Example

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_filters(mut commands: Commands) {
    commands.spawn(BeverlyToolbar::new()).with_children(|toolbar| {
        toolbar.spawn(BeverlySelect::new("Status").options(["All", "Healthy", "Warning", "Critical"]));
        toolbar.spawn(BeverlySearch::new("Search records"));
    });
}
```

## Guidance

Filters should be easy to discover, easy to change, and easy to reset. The active values should be visible and stable so the user always knows which subset of data they are looking at.
