# Container

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

Containers constrain content width and align content within a page. They are the most fundamental layout primitive for building readable screens without letting content stretch unpredictably across the viewport.

## When to use

Use a container when you need to:

- lock content to a reasonable width
- align panels and sections consistently
- keep dense screens readable without full-width chaos
- create a stable shell for cards, forms, or dashboards

## Example

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_container(mut commands: Commands) {
    commands.spawn(Node::default()).with_children(|container| {
        column()
            .header(text("Workspace"))
            .body(text("A central dashboard with deterministic spacing."))
            .footer(text("Last synchronized just now"))
            .spawn(container);
    });
}
```

## Header, Body, and Footer

Fluent containers created with `column()` or `row()` support `.header(...)`,
`.body(...)`, and `.footer(...)`. These render top, middle, and bottom in a
vertical layout, regardless of call order. Each accepts a single element or
an iterable; repeated calls append to that section.

When sections are supplied, even a `row()` becomes a vertical section host.
Use `row().children([...])` inside a section for horizontal content. Ordinary
container children join the body before explicit body content. The body grows
into available height, keeping the footer below it; unsized containers remain
content-sized. Without sections, existing row/column layout is unchanged.

The section entities carry `ContentSection::Header`, `ContentSection::Body`,
and `ContentSection::Footer` markers for application-specific styling.

## Design guidance

Keep container width predictable and let internal spacing do the work of expression. The goal is clarity and rhythm, not heavy decoration. A well-sized container helps the user focus on the content rather than the screen boundaries.
