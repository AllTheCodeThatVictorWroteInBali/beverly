# Examples Overview

The examples below are intentionally more complete than minimal snippets. Each pattern includes an application shell, a realistic hierarchy of containers and widgets, and a likely composition flow for a Bevy app that uses Beverly for a full screen or panel-based experience.

## Shared setup pattern

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BeverlyPlugin)
        .add_systems(Startup, build_shell)
        .run();
}

fn build_shell(mut commands: Commands) {
    commands.spawn((
        NodeBundle::default(),
        BeverlyAppShell::new(),
        BeverlyTheme::dark(),
    )).with_children(|parent| {
        parent.spawn(BeverlySidebar::new())
            .with_children(|sidebar| {
                sidebar.spawn(BeverlyNavItem::new("Overview").active(true));
                sidebar.spawn(BeverlyNavItem::new("Reports"));
                sidebar.spawn(BeverlyNavItem::new("Settings"));
            });

        parent.spawn(BeverlyMainPane::new()).with_children(|main| {
            main.spawn(BeverlyHeader::new("Workspace"));
            main.spawn(BeverlyStack::vertical());
        });
    });
}
```

## Pattern 1: dashboard

A dashboard combines summary cards, discrete actions, and dense data surfaces while keeping the hierarchy readable and the spacing predictable.

## Pattern 2: media app

A media app emphasizes large visual content, compact controls, and a layered translucent shell. It is a strong example of when to use glass, blur, and elevated states without overusing them.

## Pattern 3: data app

Data-heavy apps rely on high-density tables, filters, and navigation controls. The key is to preserve scanability and reduce decision friction.

## Pattern 4: AI assistant shell

AI interfaces combine prompts, status surfaces, alerts, and action controls. A robust design makes trust and state visible with secondary details rather than only flashy motion.

In every case, the idea is the same: start with an app shell, layer the screen hierarchy, then add controls and effects only where they make the task easier and the interface more understandable.
