# Data App

Data-heavy apps rely on tables, pagination, filters, and strong information hierarchy. Good data UIs reduce visual noise while giving the user as much control as they need to inspect and act on the information.

## Example

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn setup_data_app(mut commands: Commands) {
    commands.spawn((
        NodeBundle::default(),
        BeverlyDataShell::new(),
        BeverlyTheme::dark(),
    )).with_children(|parent| {
        parent.spawn(BeverlyToolbar::new()).with_children(|toolbar| {
            toolbar.spawn(BeverlySearch::new("Search records"));
            toolbar.spawn(BeverlySelect::new("Status").options(["All", "Healthy", "Warning", "Critical"]));
            toolbar.spawn(BeverlyButton::primary("Add record"));
        });

        parent.spawn(BeverlyTable::new()
            .column("Name")
            .column("Region")
            .column("Status")
            .column("Updated"));

        parent.spawn(BeverlyPagination::new(1, 8)
            .current_page(3)
            .total_pages(8));
    });
}
```

## Principles

- prioritize scanability over decorative richness
- maintain consistent column spacing and value alignment
- use filters and sorting instead of forcing users to search manually
- keep rows compact and predictable so users can compare large sets quickly
- keep the current filtered state visible and easy to undo

## Interaction details

The most reliable pattern is a small set of highly discoverable controls at the top of the table, with the dense content beneath it. This keeps the workspace responsive and gives users contextual actions without burying the underlying data.
