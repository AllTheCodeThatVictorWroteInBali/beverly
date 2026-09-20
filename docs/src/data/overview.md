# Data Overview

Beverly supports data-heavy interfaces with predictable presentation and interaction patterns. Data views are not just a place for raw information; they are the primary workspace for comparison, filtering, sorting, and action-taking.

## Core design principles

- keep dense interfaces readable without losing signal
- preserve strong hierarchy between headers, values, controls, and actions
- make filtering and sorting obvious and reversible
- reduce visual noise while maintaining clear affordances
- keep key actions close to the data they operate on

## Typical data surfaces

The data system in Beverly is designed around a set of common patterns:

- lists for simple sequential content
- tables for structured comparison
- filters for narrowing large collections
- sorting for user-controlled organization
- pagination or virtualization for very large sets
- streaming updates for live or near-real-time data

## Example: searchable data shell

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_data_shell(mut commands: Commands) {
    commands.spawn((
        NodeBundle::default(),
        BeverlyDataShell::new(),
    )).with_children(|shell| {
        shell.spawn(BeverlyToolbar::new()).with_children(|toolbar| {
            toolbar.spawn(BeverlySearch::new("Search records"));
            toolbar.spawn(BeverlySelect::new("Status").options(["All", "Healthy", "Warning", "Critical"]));
            toolbar.spawn(BeverlyButton::primary("Add record"));
        });

        shell.spawn(BeverlyTable::new()
            .column("Name")
            .column("Status")
            .column("Updated"));
    });
}
```

## Design guidance

The most effective data interfaces do not simply show more information; they make important information easier to compare and act on. A good data surface feels ordered, stable, and legible even when it contains dense rows or live updates.
