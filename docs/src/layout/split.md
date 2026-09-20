# Split

Split layouts divide a surface into primary and secondary panels. They are useful in editor-style workspaces, settings pages, and dashboard shells where one area requires focus while another remains contextually available.

## When to use

Use a split layout when:

- the user needs to compare two content areas at once
- a list or navigation panel remains visible while a details panel updates
- a primary workspace needs a persistent secondary context

## Example

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_split_panel(mut commands: Commands) {
    commands.spawn((
        NodeBundle::default(),
        BeverlySplit::horizontal(),
    )).with_children(|split| {
        split.spawn(BeverlySidebar::new());
        split.spawn(BeverlyMainPane::new());
    });
}
```

## Guidance

Keep the split proportion stable and ensure each pane still has enough breathing room. A split layout should not become a cramped crowding mechanism for unrelated content.
