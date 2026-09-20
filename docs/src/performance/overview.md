# Performance Overview

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

Performance is an architectural requirement for Bevy-based user interfaces. In a UI framework, performance is not just about frames; it is about how much state churn, layout work, and asset cost the system creates while users work with the interface.

## Design principles

- keep updates local and predictable
- limit unnecessary re-render and re-layout work
- avoid expensive pattern growth in large lists or dense tables
- preserve a stable layout while data changes underneath it
- keep rendering and interaction cost proportional to the actual user task

## Example: constrained list workload

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_data_view(mut commands: Commands) {
    commands.spawn((
        NodeBundle::default(),
        BeverlyList::new(),
    )).with_children(|list| {
        for i in 0..50 {
            list.spawn(BeverlyListItem::new(format!("Item {i}")));
        }
    });
}
```

This keeps the content model explicit and understandable. For larger workloads, virtualization or paginated patterns are better than rendering every item with equal visual weight.

## Guidance

Performance decisions should be visible in the architecture. A user interface that feels fast should also scale well when the data model grows, the screen becomes denser, or motion and effects become more complex.
