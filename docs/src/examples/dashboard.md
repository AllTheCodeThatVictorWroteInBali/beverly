# Dashboard

Dashboards are density-heavy views that benefit from structured cards, badges, tables, and clear action areas. The pattern is to keep the content architecture calm while the data and controls are rich.

## Example

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn setup_dashboard(mut commands: Commands) {
    commands.spawn((
        NodeBundle::default(),
        BeverlyDashboard::new(),
        BeverlyTheme::dark(),
    )).with_children(|parent| {
        parent.spawn(BeverlyToolbar::new()).with_children(|toolbar| {
            toolbar.spawn(BeverlySearch::new("Search workspace"));
            toolbar.spawn(BeverlyButton::secondary("Export CSV"));
            toolbar.spawn(BeverlyButton::primary("Create report"));
        });

        parent.spawn(BeverlyMetricGrid::new()).with_children(|grid| {
            grid.spawn(BeverlyMetricCard::new("Revenue", "$84.2k")
                .delta("+12.5%")
                .trend(Trend::Positive));
            grid.spawn(BeverlyMetricCard::new("Active tasks", "18")
                .delta("6 due today")
                .trend(Trend::Neutral));
            grid.spawn(BeverlyMetricCard::new("Open incidents", "3")
                .delta("1 critical")
                .trend(Trend::Warning));
        });

        parent.spawn(BeverlyContentRow::new()).with_children(|row| {
            row.spawn(BeverlyPanel::new("Operations").with_content(|panel| {
                panel.spawn(BeverlyTable::new()
                    .column("Name")
                    .column("Owner")
                    .column("Status"));
            }));

            row.spawn(BeverlyPanel::new("Highlights").with_content(|panel| {
                panel.spawn(BeverlyBadge::new("Live").variant(BadgeVariant::Success));
                panel.spawn(BeverlyAlert::new("Two actions need review")
                    .variant(AlertVariant::Warning));
            }));
        });
    });
}
```

## Why this pattern works

- the top toolbar keeps high-value actions visible without crowding the main content
- metric cards promote scannability and quick comparison
- the table stays dense but predictable
- alerts and status badges provide useful context without overwhelming the page

## Design guidance

- keep visual density proportional to user priority
- use color and badge semantics to emphasize state, not just decoration
- leave large whitespace around the most important metrics or actions
- use consistent action ordering across dashboard surfaces
