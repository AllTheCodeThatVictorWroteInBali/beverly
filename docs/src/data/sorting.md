# Sorting

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

Sorting tools help users reorganize information by clear criteria. Sorting is one of the most valuable ways to reduce friction in a data-heavy surface when the user wants to compare values or find outliers quickly.

## Example

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_sort_controls(mut commands: Commands) {
    commands.spawn(BeverlyToolbar::new()).with_children(|toolbar| {
        toolbar.spawn(BeverlySelect::new("Sort by")
            .options(["Newest", "Oldest", "Priority", "Name"]));
    });
}
```

## Guidance

Sorting should be explicit and easy to reverse. If the current sort order matters, it should be visible and understandable to the user at a glance.
