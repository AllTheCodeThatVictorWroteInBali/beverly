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
    commands.spawn((
        NodeBundle::default(),
        BeverlyContainer::new(),
    )).with_children(|container| {
        container.spawn(BeverlyTitle::new("Workspace"));
        container.spawn(BeverlyText::new("A central dashboard with deterministic spacing."));
    });
}
```

## Design guidance

Keep container width predictable and let internal spacing do the work of expression. The goal is clarity and rhythm, not heavy decoration. A well-sized container helps the user focus on the content rather than the screen boundaries.
