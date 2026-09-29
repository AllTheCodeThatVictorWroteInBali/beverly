# Navbar

A Navbar is a horizontal layout primitive for organizing content across the top or edge of an interface.

It provides three named sections:

- `left`
- `center`
- `right`

The center section is always centered within the Navbar. The left and right sections work inward from their respective edges.

The Navbar can also contain arbitrary `children()`, making it useful for simple layouts as well as more structured application navigation.

**The Navbar provides the layout. The application provides the navigation.**

## Basic Usage

Create a Navbar with `navbar()` and compose content into its sections:

```rust id="d8q3m1"
navbar()
    .left([
        text("My App"),
    ])
    .center([
        text("Dashboard"),
    ])
    .right([
        text("Account"),
    ])
```

The three sections have predictable spatial behavior:

```text
┌──────────────────────────────────────────────────────────┐
│  LEFT                  CENTER                  RIGHT      │
│  ←───────              ───────              ───────→    │
└──────────────────────────────────────────────────────────┘
```

The center section remains centered in the Navbar rather than simply appearing after the left section.

This makes the layout predictable even when the left and right sections contain different amounts of content.

## Named Sections

The three named sections are semantic conveniences for the most common Navbar layout.

### Left

`left()` places content against the left side of the Navbar.

```rust id="p4h8x2"
navbar()
    .left([
        button(localize("navigation.menu"))
            .label(localize("navigation.menu.label"))
            .aria(localize("navigation.menu.description")),

        text("My App"),
    ])
```

The left section grows inward from the edge toward the center.

### Center

`center()` places content in the center of the Navbar.

```rust id="j7k2w9"
navbar()
    .center([
        text("Dashboard"),
    ])
```

The center section is centered relative to the Navbar itself.

It does not simply follow the left section.

This distinction is important when building predictable application layouts.

### Right

`right()` places content against the right side of the Navbar.

```rust id="v5n1r6"
navbar()
    .right([
        button(localize("user.profile"))
            .label(localize("user.profile.label"))
            .aria(localize("user.profile.description")),
    ])
```

The right section grows inward from the right edge toward the center.

## Using All Three

A common application Navbar might look like this:

```rust id="a2m8q4"
navbar()
    .left([
        button(localize("navigation.menu"))
            .label(localize("navigation.menu.label"))
            .aria(localize("navigation.menu.description")),

        text("Beverly"),
    ])
    .center([
        text("Dashboard"),
    ])
    .right([
        button(localize("user.profile"))
            .label(localize("user.profile.label"))
            .aria(localize("user.profile.description")),
    ])
```

The resulting structure is conceptually:

```text
LEFT CONTENT        CENTER CONTENT        RIGHT CONTENT
     →                    ●                    ←
```

The center remains centered even as the left and right content changes.

## Children

`children()` remains the general composition primitive.

```rust id="r6c3y8"
navbar()
    .children([
        text("Beverly"),
        text("Dashboard"),
    ])
```

This is useful when the application does not need the three-section layout.

Named sections are conveniences, not a replacement for `children()`.

A Navbar can therefore be used as either a simple container:

```rust id="u9p4k1"
navbar()
    .children([
        text("My Application"),
    ])
```

or as a structured three-section layout:

```rust id="x3f7m2"
navbar()
    .left(text("Beverly"))
    .center(text("Dashboard"))
    .right(text("Account"))
```

A section can accept one child or many children.

```rust id="b8q2v5"
navbar()
    .left(text("Beverly"))
```

```rust id="n4k6r1"
navbar()
    .left([
        icon("menu"),
        text("Beverly"),
    ])
```

**`children()` is the general primitive. Named sections add useful semantics.**

## Fixed and Non-Fixed Navbars

A Navbar can either participate in normal layout flow or remain fixed relative to its containing interface.

For a normal Navbar:

```rust id="m2w7c9"
navbar()
    .left(text("Beverly"))
    .center(text("Dashboard"))
    .right(text("Account"))
```

For a fixed Navbar:

```rust id="q5j1v8"
navbar()
    .fixed(true)
    .left(text("Beverly"))
    .center(text("Dashboard"))
    .right(text("Account"))
```

A fixed Navbar remains positioned according to the application's layout rules rather than moving with ordinary content flow.

The Navbar itself does not decide whether an application should use fixed positioning. It simply provides the layout option.

## Styling

Navbar uses the same fluent styling model as other Beverly components.

```rust id="k8r3p6"
navbar()
    .padding(16)
    .height(64)
    .radius(12)
    .left(text("Beverly"))
    .center(text("Dashboard"))
    .right(text("Account"))
```

Layout and presentation remain separate from application behavior.

The Navbar determines where content goes. Its children determine what that content is.

## Navigation

Despite its name, the Navbar does not own application navigation.

Navigation is application behavior.

A navigation control can emit a UI Event or connect directly to an application Command:

```rust id="t6y2m4"
navbar()
    .left([
        button(localize("navigation.home"))
            .label(localize("navigation.home.label"))
            .aria(localize("navigation.home.description"))
            .on("click", Command::Navigation::Home),

        button(localize("navigation.settings"))
            .label(localize("navigation.settings.label"))
            .aria(localize("navigation.settings.description"))
            .on("click", Command::Navigation::Settings),
    ])
```

The Navbar does not need to know what `Command::Navigation::Home` does.

It only provides the surface in which the controls are arranged.

**The Navbar provides the layout. Buttons provide the interaction. Commands provide the application behavior.**

## Responsive Content

Because the Navbar is composed from ordinary components, its contents can be data-driven.

```rust id="w1c7n5"
navbar()
    .left(navigation_menu())
    .center(text(page.title()))
    .right(user_actions())
```

Higher-level pieces are ordinary Rust functions:

```rust id="z4p8q2"
fn user_actions() -> impl Component {
    row()
        .children([
            button(localize("user.notifications"))
                .label(localize("user.notifications.label"))
                .aria(localize("user.notifications.description")),

            avatar()
                .src(user.avatar_url())
                .aria(localize("user.avatar.description")),
        ])
}
```

There is no separate Navbar component language.

**Higher-level components are ordinary Rust functions that return lower-level components.**

## Accessibility

The Navbar is a layout primitive. Its children remain responsible for their own interaction semantics.

Interactive elements inside the Navbar should have their own accessible labels and descriptions:

```rust id="e7m3k6"
navbar()
    .left([
        button(localize("navigation.menu"))
            .label(localize("navigation.menu.label"))
            .aria(localize("navigation.menu.description")),
    ])
    .center([
        text("Dashboard"),
    ])
```

The Navbar itself can also expose accessible semantics when the application needs the navigation surface to be identified:

```rust id="h2v9r4"
navbar()
    .label(localize("navigation.primary.label"))
    .aria(localize("navigation.primary.description"))
    .left([
        text("Beverly"),
    ])
    .center([
        text("Dashboard"),
    ])
```

The exact semantic role belongs to the Navbar's implementation and context; applications should not need to manually reproduce the underlying layout or accessibility mechanics.

**Accessible by construction, not accessible by cleanup.**

## What Navbar Does Not Do

Navbar does not:

- own navigation state
- decide which page is active
- manage routing
- own application data
- define authentication
- define user permissions
- replace Buttons or Links
- require a separate navigation state system

Those concerns belong to the application.

The Navbar is responsible for organizing interface content.

## Design Principle

The Navbar deliberately provides only the structure that is difficult to express consistently by hand:

- content from the left edge
- content from the center
- content from the right edge
- predictable centered positioning
- optional fixed positioning
- general `children()` composition

Everything else remains ordinary Beverly composition and ordinary Rust.

**Three sections when you need them. `children()` when you don't.**

**The center stays centered. The edges work inward. The application owns the meaning.**
