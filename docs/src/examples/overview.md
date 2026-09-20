# Examples Overview

The examples below are intentionally more complete than minimal snippets. Each pattern includes an application shell, a realistic hierarchy of containers and widgets, and a likely composition flow for a Bevy app that uses Beverly for a full screen or panel-based experience.

These examples are not merely visual arrangements. They are practical demonstrations of the design system in real product contexts: where hierarchy should be strongest, which surfaces should absorb density, and where motion, glass, or semantics should be used to reinforce understanding instead of decoration.

<img src="../assets/example-overview.svg" alt="Example application overview illustration" width="860" />

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

This pattern establishes the same conceptual model we see throughout the book: a strong shell, a predictable hierarchy, and a small number of high-value interaction surfaces. The shell is the scaffold; the content and controls are what make the app feel intentional.

## Pattern 1: dashboard

A dashboard combines summary cards, discrete actions, and dense data surfaces while keeping the hierarchy readable and the spacing predictable. This is a good example of how Beverly treats information density as a product decision rather than just a layout preference.

## Pattern 2: media app

A media app emphasizes large visual content, compact controls, and a layered translucent shell. It is a strong example of when to use glass, blur, and elevated states without overusing them. Here the interface must feel immersive but never lose readability or control.

## Pattern 3: data app

Data-heavy apps rely on high-density tables, filters, and navigation controls. The key is to preserve scanability and reduce decision friction. Data surfaces are one of the clearest places where a calm, consistent system matters more than dramatic visual effects.

## Pattern 4: AI assistant shell

AI interfaces combine prompts, status surfaces, alerts, and action controls. A robust design makes trust and state visible with secondary details rather than only flashy motion. In AI surfaces, the system must make uncertainty, permissions, and tool usage legible.

## The architectural idea behind all examples

In every case, the idea is the same: start with an app shell, layer the screen hierarchy, then add controls and effects only where they make the task easier and the interface more understandable.

A good example does not merely show a pretty composition; it demonstrates the reasoning behind the layout, hierarchy, and interaction model. Beverly’s patterns are strongest when the interface feels stable under pressure: complex content, multiple actions, frequent updates, and changing states still remain clear to the user.

## Example composition checklist

Before a screen is considered done, ask:

- Can the key actions be reached by keyboard?
- Does focus move predictably after a panel or modal opens?
- Can the user understand the important states without relying on color alone?
- Does reduced motion still leave the interface legible and stable?
- Is the visual density justified by the task and the surface's purpose?
- Do the shell and the content feel like the same system, rather than separate visual styles?

If the answer to those questions is yes, the example is likely a strong Beverly pattern rather than just a decorative mockup.

## A useful mental model

Think of each example as a product surface that is composed from a few reusable decisions:

- shell and navigation structure
- primary content region
- status or utility surfaces
- action layer and intent cues
- decorative treatment only where it helps understanding

That model keeps the examples realistic while reinforcing the broader system philosophy: the interface is an intentional arrangement of actions, state, and meaning—not a pile of floating widgets.
