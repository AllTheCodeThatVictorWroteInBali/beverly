# Getting Started

Beverly fits naturally into a normal Bevy app. The simplest setup is to add Beverly's plugin, build a shell, and then add widgets to that shell with ECS composition.

## Minimal app shell

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BeverlyPlugin)
        .add_systems(Startup, setup_ui)
        .run();
}

fn setup_ui(mut commands: Commands) {
    commands.spawn((
        NodeBundle::default(),
        BeverlyAppShell::new(),
        BeverlyTheme::dark(),
    )).with_children(|parent| {
        parent.spawn(BeverlyCard::new("Welcome")
            .with_body("Your first Beverly screen"));
        parent.spawn(BeverlyButton::primary("Start"));
    });
}
```

## A realistic first screen pattern

Most applications benefit from a shell with a main content pane and a few high-level sections. This pattern scales from a small app to a dense workspace without forcing the UI into a different architecture.

```rust
fn setup_workspace(mut commands: Commands) {
    commands.spawn((
        NodeBundle::default(),
        BeverlyAppShell::new(),
    )).with_children(|parent| {
        parent.spawn(BeverlySidebar::new()).with_children(|sidebar| {
            sidebar.spawn(BeverlyNavItem::new("Overview").active(true));
            sidebar.spawn(BeverlyNavItem::new("Reports"));
            sidebar.spawn(BeverlyNavItem::new("Settings"));
        });

        parent.spawn(BeverlyMainPane::new()).with_children(|main| {
            main.spawn(BeverlyHeader::new("Workspace"));
            main.spawn(BeverlyStack::vertical()).with_children(|stack| {
                stack.spawn(BeverlyCard::new("Summary")
                    .with_body("Everything is running normally."));
                stack.spawn(BeverlyButton::primary("Create task"));
            });
        });
    });
}
```

## Building with the design system

1. Start with a stable app shell or layout root.
2. Add sections and panels for the major content types.
3. Place controls and data widgets inside those sections.
4. Use surfaces, shadows, gradients, and blur intentionally where the hierarchy needs emphasis.
5. Validate keyboard focus, reduced motion, and contrast before shipping.

## Next steps

- read the component catalog in [Components](./components/overview.md)
- review the visual system in [Foundations](./foundations/design-tokens.md)
- browse the example patterns in [Examples Overview](./examples/overview.md)
