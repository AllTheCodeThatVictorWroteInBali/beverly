# Data App

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

Data-heavy apps rely on tables, pagination, filters, and strong information hierarchy. Good data UIs reduce visual noise while giving the user as much control as they need to inspect and act on the information. In this kind of surface, the interface is not an aesthetic exercise; it is a workbench for reasoning.

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
- give the user direct access to the state of the dataset without making them interpret visual complexity

## Interaction details

The most reliable pattern is a small set of highly discoverable controls at the top of the table, with the dense content beneath it. This keeps the workspace responsive and gives users contextual actions without burying the underlying data.

A data app should feel like a controlled system, not like a visually crowded document. The user should be able to quickly answer questions such as: what is visible, what is filtered, what is selected, and which actions are available.

## Why density is a design problem

Dense tables are not a visual challenge alone; they are a cognitive one. When a user scans a large dataset, the interface must support quick comparison across rows and columns without encouraging misreading. This is why alignment, typography, spacing, and contrast are so important in data design.

A table with strong structure is easier to trust than a table with arbitrary color or decorative states. Beverly’s data pattern encourages calm semantics: status chips, focus states, consistent ordering, and predictable sorting behavior.

## Recommended product behavior

Good data apps make context visible:

- filter state is visibly active
- sort order is easy to understand
- rows have an obvious selection model
- unsafe or irreversible actions are confirmable
- keyboard navigation feels exact and stable

If those conditions hold, the data view can absorb a great deal of information without becoming overwhelming.
