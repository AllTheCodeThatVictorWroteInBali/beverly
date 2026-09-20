# Lists

Lists represent ordinal collections with compact reading patterns. They are the simplest way to show ordered information, actions, or grouped content when tables would be too heavy for the task.

## Use cases

Use a list when a user is scanning a sequence of similar items, such as:

- notifications
- search results
- tasks or jobs
- workspace members
- user actions or recent events

## Example

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_list(mut commands: Commands) {
    commands.spawn((
        NodeBundle::default(),
        BeverlyList::new(),
    )).with_children(|list| {
        list.spawn(BeverlyListItem::new("Daily summary"));
        list.spawn(BeverlyListItem::new("Migration status"));
        list.spawn(BeverlyListItem::new("Release notes"));
    });
}
```

## Guidance

Lists should read cleanly and be easy to scan. Keep each item consistent in structure and density, and pair them with strong labels or metadata when the items need context.
