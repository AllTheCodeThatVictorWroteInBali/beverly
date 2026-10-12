# List Item

A list item presents a title with an optional subtitle and icon in a full-width
row. `BeverlyPlugin` installs the visual system, which follows the active theme
and interaction state.

## Live Demo

<iframe src="../../assets/demos/index.html?demo=list_item" title="Interactive Beverly list item demo" loading="lazy" allow="fullscreen" style="display: block; width: 100%; height: min(820px, 90vh); min-height: 580px; border: 0;"></iframe>

<a href="../../assets/demos/index.html?demo=list_item" target="_blank" rel="noopener">Open the demo in a separate tab</a>.

## Basic Usage

```rust
use beverly::components::list_item::spawn_list_item;
use beverly::icons::Icon;
use bevy::prelude::*;

fn add_item(parent: &mut ChildSpawnerCommands) -> Entity {
    spawn_list_item(
        parent,
        "Inbox",
        Some("Three unread messages".to_string()),
        Some(Icon::feather("mail")),
    )
}
```

Pass `None` to omit the subtitle or icon. Arrange multiple items in a parent
`Node` with `FlexDirection::Column` and the desired row gap.

The returned entity carries `ListItem` and Bevy's `Button` marker. Selection,
navigation, and application actions are not assigned by this helper; attach
your own marker and handle interaction in your application.