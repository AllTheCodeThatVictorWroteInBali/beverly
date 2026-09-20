# Dropdown

Dropdowns reveal a contextual menu of actions or choices without permanently taking up screen space. They are useful when a compact control needs to surface a set of options or actions without overwhelming the surrounding layout.

## Example

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_menu(mut commands: Commands) {
    commands.spawn(BeverlyDropdown::new("Actions")
        .item("Rename")
        .item("Duplicate")
        .item("Archive"));
}
```

## Best practices

- keep action menus focused and short
- ensure destructive actions are explicit
- close the menu when the user chooses an item or clicks outside the control
- support keyboard navigation and focus return
- keep the trigger label clear enough to understand the menu's purpose

## Accessibility

Dropdowns should preserve an obvious focus state and predictable keyboard behavior. They must support both pointer and keyboard interaction and should close cleanly without leaving the user stranded in a hidden or orphaned menu context.
