# Input

Input treatment covers both pointer and keyboard interaction semantics. Beverly should treat input as a shared interaction layer rather than a collection of random widget behaviors.

## Interaction principles

- pointer and keyboard input should have equivalent outcomes where possible
- focus should be stable and visible during editing
- input areas should provide immediate and legible feedback
- interaction rules should be consistent across text, selects, search boxes, and forms

## Example

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_input_surface(mut commands: Commands) {
    commands.spawn((
        NodeBundle::default(),
        BeverlyStack::vertical(),
    )).with_children(|stack| {
        stack.spawn(BeverlyInput::new("Query").placeholder("Search for files"));
        stack.spawn(BeverlyButton::primary("Run search"));
    });
}
```

## Guidance

Input surfaces should feel consistent and reliable. The user should not have to learn different interaction rules for each field type, and the app should support both direct editing and accessible keyboard workflows.
