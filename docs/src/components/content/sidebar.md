# Sidebar

A Sidebar is a layout primitive for organizing content along the side of an interface.

A Sidebar can be positioned on either the **left** or **right** side of its containing layout.

Like the Navbar and Footer, it supports general `children()` composition and does not impose a separate application structure.

**The Sidebar provides the layout. The application provides the content and behavior.**

## Basic Usage

Create a Sidebar with `sidebar()` and place it on the left:

```rust id="p7m3q8"
sidebar()
    .side(Side::Left)
    .children([
        text("Navigation"),
        text("Dashboard"),
        text("Settings"),
    ])
```

Or place it on the right:

```rust id="v4k8n2"
sidebar()
    .side(Side::Right)
    .children([
        text("Details"),
        text("Properties"),
    ])
```

The side is an explicit configuration choice:

```rust id="x2r6m9"
sidebar()
    .side(Side::Left)
```

```rust id="n8q1v5"
sidebar()
    .side(Side::Right)
```

**One component. Two sides. Explicit configuration.**

## Children

`children()` is the fundamental composition primitive.

```rust id="c5w9k3"
sidebar()
    .side(Side::Left)
    .children([
        text("Dashboard"),
        text("Projects"),
        text("Settings"),
    ])
```

A Sidebar can contain one child or many:

```rust id="h6p2r8"
sidebar()
    .side(Side::Left)
    .children(text("Navigation"))
```

```rust id="j3m7q1"
sidebar()
    .side(Side::Left)
    .children([
        icon("home"),
        text("Dashboard"),
        text("Projects"),
        text("Settings"),
    ])
```

There is no special Sidebar composition system.

**`children()` is the general primitive.**

## Left and Right Positioning

The Sidebar works from the edge of the containing layout inward.

A left Sidebar:

```text id="m8v4q2"
┌──────────────┬──────────────────────────────────────────┐
│              │                                          │
│   SIDEBAR    │              MAIN CONTENT                │
│     ←        │                                          │
│              │                                          │
└──────────────┴──────────────────────────────────────────┘
```

A right Sidebar:

```text id="r5k1n7"
┌──────────────────────────────────────────┬──────────────┐
│                                          │              │
│              MAIN CONTENT                │   SIDEBAR →  │
│                                          │              │
│                                          │              │
└──────────────────────────────────────────┴──────────────┘
```

The application does not need separate components for these two layouts.

```rust id="q9w3m6"
sidebar()
    .side(Side::Left)
```

and:

```rust id="z4p8k2"
sidebar()
    .side(Side::Right)
```

## Fixed and Non-Fixed Sidebars

A Sidebar can either participate in normal layout flow or remain fixed relative to its containing interface.

A normal Sidebar:

```rust id="b6n2r9"
sidebar()
    .side(Side::Left)
    .children(navigation_menu())
```

A fixed Sidebar:

```rust id="s8q4m1"
sidebar()
    .side(Side::Left)
    .fixed(true)
    .children(navigation_menu())
```

The same applies to a right Sidebar:

```rust id="t3v7k5"
sidebar()
    .side(Side::Right)
    .fixed(true)
    .children(details_panel())
```

The Sidebar provides the positioning option. The application decides whether the Sidebar should be fixed.

## Styling

Sidebar uses the same fluent styling model as other Beverly components.

```rust id="k7m2q8"
sidebar()
    .side(Side::Left)
    .width(240)
    .padding(16)
    .children(navigation_menu())
```

A right Sidebar can use the same API:

```rust id="f4r9v3"
sidebar()
    .side(Side::Right)
    .width(320)
    .padding(16)
    .children(details_panel())
```

The side does not change the composition model.

## Interactive Content

A Sidebar is not inherently interactive.

It commonly contains interactive components such as Buttons or other navigation controls:

```rust id="u8p3m6"
sidebar()
    .side(Side::Left)
    .children([
        button().text(localize("navigation.home"))
            .on("click", Command::Navigation::Home),

        button().text(localize("navigation.settings"))
            .on("click", Command::Navigation::Settings),
    ])
```

The Sidebar does not need to know what those commands mean.

It simply provides the space in which the controls are arranged.

**The Sidebar provides the layout. The children provide the interaction. The application provides the behavior.**

## Data-Driven Sidebars

Sidebar content can come directly from application data:

```rust id="w5n1q7"
sidebar()
    .side(Side::Left)
    .children(
        navigation_items()
    )
```

A higher-level component can be an ordinary Rust function:

```rust id="e2m8r4"
fn navigation_items() -> impl Component {
    column()
        .children([
            navigation_item("Dashboard", Command::Navigation::Dashboard),
            navigation_item("Projects", Command::Navigation::Projects),
            navigation_item("Settings", Command::Navigation::Settings),
        ])
}
```

The Sidebar does not need to know how those items are generated.

**Higher-level components are ordinary Rust functions that return lower-level components.**

## Accessibility

A Sidebar can expose accessible semantics when it represents a meaningful region of the interface:

```rust id="a9q3v6"
sidebar()
    .side(Side::Left)
    .label(localize("navigation.sidebar.label"))
    .aria(localize("navigation.sidebar.description"))
    .children(navigation_menu())
```

Interactive elements inside the Sidebar remain responsible for their own accessible labels and descriptions:

```rust id="g7m2p5"
sidebar()
    .side(Side::Left)
    .children([
        button().text(localize("navigation.home"))
            .on("click", Command::Navigation::Home),
    ])
```

The Sidebar should provide the appropriate structural semantics without requiring the application to manually reproduce the underlying accessibility behavior.

**Accessible by construction, not accessible by cleanup.**

## Sidebar as Navigation

A Sidebar is often used for navigation, but navigation is not its responsibility.

The same Sidebar can contain navigation:

```rust id="q4n8m2"
sidebar()
    .side(Side::Left)
    .children(navigation_menu())
```

or application information:

```rust id="y6p1r7"
sidebar()
    .side(Side::Right)
    .children([
        text("Document Details"),
        text(document.title()),
        badge(document.status()),
    ])
```

or tools:

```rust id="c8v3k5"
sidebar()
    .side(Side::Right)
    .children(tool_panel())
```

The layout primitive stays the same.

**A Sidebar does not have to be navigation. It is simply content positioned at the side of an interface.**

## What Sidebar Does Not Do

Sidebar does not:

- own navigation state
- manage routing
- decide which page is active
- own application data
- define permissions
- define authentication
- require a separate navigation system
- dictate what content belongs inside it

Those concerns belong to the application.

The Sidebar is responsible for positioning and composing side content.

## Design Principle

The Sidebar deliberately provides only the structure applications repeatedly need:

- left or right positioning
- predictable edge alignment
- optional fixed positioning
- general `children()` composition
- fluent styling

Everything else remains ordinary Beverly composition and ordinary Rust.

**Left or right when you need it. `children()` when you don't need anything more.**

**The Sidebar provides the space. The application gives the space meaning.**
