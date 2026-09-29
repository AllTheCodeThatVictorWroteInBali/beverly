# Footer

A Footer is a layout primitive for organizing content along the bottom of an interface.

It follows the same composition model as the Navbar, providing three named sections:

- `left`
- `center`
- `right`

The center section is always centered within the Footer. The left and right sections work inward from their respective edges.

The Footer can also contain arbitrary `children()`, making it useful for simple layouts as well as structured application footers.

**The Footer provides the layout. The application provides the content.**

## Basic Usage

Create a Footer with `footer()` and compose content into its sections:

```rust id="f7k2m4"
footer()
    .left([
        text("© 2026 Beverly"),
    ])
    .center([
        text("Built with Rust"),
    ])
    .right([
        text("v1.0.0"),
    ])
```

The three sections have predictable spatial behavior:

```text id="r3p8w1"
┌──────────────────────────────────────────────────────────┐
│  LEFT                  CENTER                  RIGHT      │
│  ←───────              ───────              ───────→    │
└──────────────────────────────────────────────────────────┘
```

The center remains centered relative to the Footer rather than simply appearing after the left section.

## Named Sections

The three named sections are semantic conveniences for common Footer layouts.

### Left

`left()` places content against the left side of the Footer.

```rust id="m6q2v9"
footer()
    .left([
        text("© 2026 Beverly"),
        text("All rights reserved"),
    ])
```

The left section grows inward from the edge toward the center.

### Center

`center()` places content in the center of the Footer.

```rust id="c4n8r2"
footer()
    .center([
        text("Built with Rust"),
    ])
```

The center section is centered relative to the Footer itself.

It does not simply follow the left section.

### Right

`right()` places content against the right side of the Footer.

```rust id="p9w3k7"
footer()
    .right([
        button(localize("footer.settings"))
            .label(localize("footer.settings.label"))
            .aria(localize("footer.settings.description")),
    ])
```

The right section grows inward from the right edge toward the center.

## Using All Three

A typical application Footer might look like this:

```rust id="v5m1q8"
footer()
    .left([
        text("© 2026 Beverly"),
    ])
    .center([
        text("Privacy"),
        text("Terms"),
    ])
    .right([
        text("v1.0.0"),
    ])
```

The center remains centered even when the left and right sections contain different amounts of content.

This makes Footer layouts predictable without requiring application-specific positioning logic.

## Children

`children()` remains the general composition primitive.

```rust id="x8q3m6"
footer()
    .children([
        text("© 2026 Beverly"),
        text("All rights reserved"),
    ])
```

This is useful when the application does not need the three-section layout.

Named sections are conveniences, not a replacement for `children()`.

A Footer can therefore be used as a simple container:

```rust id="j4r7p2"
footer()
    .children([
        text("© 2026 Beverly"),
    ])
```

or as a structured three-section layout:

```rust id="n6v2k9"
footer()
    .left(text("© 2026 Beverly"))
    .center(text("Privacy"))
    .right(text("v1.0.0"))
```

A section can accept one child or many children:

```rust id="q3m8w5"
footer()
    .left(text("Beverly"))
```

```rust id="t7p1r4"
footer()
    .left([
        icon("copyright"),
        text("2026 Beverly"),
    ])
```

**`children()` is the general primitive. Named sections add useful semantics.**

## Fixed and Non-Fixed Footers

A Footer can either participate in normal layout flow or remain fixed relative to its containing interface.

For a normal Footer:

```rust id="b9k4m2"
footer()
    .left(text("© 2026 Beverly"))
    .center(text("Privacy"))
    .right(text("v1.0.0"))
```

For a fixed Footer:

```rust id="s5q8n1"
footer()
    .fixed(true)
    .left(text("© 2026 Beverly"))
    .center(text("Privacy"))
    .right(text("v1.0.0"))
```

A fixed Footer remains positioned according to the application's layout rules rather than moving with ordinary content flow.

The Footer does not decide whether fixed positioning is appropriate. It simply provides the option.

## Styling

Footer uses the same fluent styling model as other Beverly components.

```rust id="k2v6r9"
footer()
    .padding(16)
    .height(64)
    .radius(12)
    .left(text("© 2026 Beverly"))
    .center(text("Privacy"))
    .right(text("v1.0.0"))
```

Layout and presentation remain separate from application behavior.

The Footer determines where content goes. Its children determine what that content is.

## Interactive Content

A Footer is not inherently interactive.

It can contain Buttons and other interactive components when the application needs them:

```rust id="w4m7q2"
footer()
    .left([
        text("© 2026 Beverly"),
    ])
    .right([
        button(localize("footer.settings"))
            .label(localize("footer.settings.label"))
            .aria(localize("footer.settings.description"))
            .on("click", Command::Settings::Open),
    ])
```

The Footer does not need to know what the Command does.

It simply provides the layout in which the control appears.

**The Footer provides the layout. The children provide the interaction. The application provides the behavior.**

## Data-Driven Footers

Footer content can come directly from application data:

```rust id="e8p3m6"
footer()
    .left(text(app.copyright()))
    .center(footer_links())
    .right(text(app.version()))
```

Higher-level pieces remain ordinary Rust functions:

```rust id="u5r1k7"
fn footer_links() -> impl Component {
    row()
        .children([
            button(localize("footer.privacy"))
                .label(localize("footer.privacy.label"))
                .aria(localize("footer.privacy.description"))
                .on("click", Command::Navigation::Privacy),

            button(localize("footer.terms"))
                .label(localize("footer.terms.label"))
                .aria(localize("footer.terms.description"))
                .on("click", Command::Navigation::Terms),
        ])
}
```

There is no separate Footer component language.

**Higher-level components are ordinary Rust functions that return lower-level components.**

## Accessibility

The Footer is primarily a layout primitive. Interactive elements inside it remain responsible for their own accessible semantics.

```rust id="a7n4q2"
footer()
    .left([
        text("© 2026 Beverly"),
    ])
    .right([
        button(localize("footer.settings"))
            .label(localize("footer.settings.label"))
            .aria(localize("footer.settings.description")),
    ])
```

The Footer can also expose accessible semantics when the application needs the footer region to be identified:

```rust id="z2m8v5"
footer()
    .label(localize("footer.label"))
    .aria(localize("footer.description"))
    .left([
        text("© 2026 Beverly"),
    ])
    .center([
        text("Privacy"),
    ])
    .right([
        text("v1.0.0"),
    ])
```

The underlying semantic and layout behavior should be handled by Beverly rather than requiring developers to manually reproduce it.

**Accessible by construction, not accessible by cleanup.**

## What Footer Does Not Do

Footer does not:

- own application state
- manage navigation state
- manage routing
- own application data
- define permissions
- define authentication
- require a separate layout system
- determine what footer content means

Those concerns belong to the application.

The Footer is responsible for organizing interface content.

## Navbar and Footer

Navbar and Footer intentionally share the same basic layout vocabulary.

| Component  | Purpose                                                 |
| ---------- | ------------------------------------------------------- |
| `navbar()` | Organizes content at the top or edge of an interface    |
| `footer()` | Organizes content at the bottom or edge of an interface |

Both provide:

- `children()`
- `left()`
- `center()`
- `right()`
- predictable centered positioning
- optional fixed positioning
- fluent styling
- ordinary Rust composition

The difference is semantic placement, not architectural complexity.

**Same composition model. Different place in the interface.**

## Design Principle

The Footer deliberately provides only the structure that applications repeatedly need:

- content from the left edge
- content from the center
- content from the right edge
- predictable centered positioning
- optional fixed positioning
- general `children()` composition

Everything else remains ordinary Beverly composition and ordinary Rust.

**Three sections when you need them. `children()` when you don't.**

**The center stays centered. The edges work inward. The application owns the meaning.**
