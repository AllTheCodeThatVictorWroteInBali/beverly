# Dashboard

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

Dashboards are density-heavy views that benefit from structured cards, badges, tables, and clear action areas. The pattern is to keep the content architecture calm while the data and controls are rich. In a product context, a dashboard is not just a collection of metrics; it is a command surface for making decisions under time pressure.

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
- the layout gives each region a distinct purpose: action, overview, detail

This is the essence of a strong dashboard: information density is controlled by structure, not just by shrinking everything down into a noisy grid.

## Design guidance

- keep visual density proportional to user priority
- use color and badge semantics to emphasize state, not just decoration
- leave large whitespace around the most important metrics or actions
- use consistent action ordering across dashboard surfaces
- prefer a few high-signal summary cards over too many low-value widgets

## Layout logic

The dashboard pattern works because the eye has a clear path through the interface:

1. action area at the top
2. summary cards as the first read
3. data and operational detail beneath it
4. contextual alerts or side information at the edge of the flow

This keeps the experience useful even when the screen contains a lot of information. The system does not force the user to interpret everything equally; it helps them understand what matters most first.

## Interaction model

Dashboards usually live at the intersection of monitoring and action. That means the controls should be readable at a glance and safe to interact with even under cognitive load. Search, filter, and export actions should stay obvious, while the dense data region can remain compact and information-rich.

A dashboard fails when it looks powerful but becomes hard to reason about. Beverly’s approach is to elevate the structure around the data rather than decorating the data itself.
