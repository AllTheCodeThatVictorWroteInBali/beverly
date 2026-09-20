# Navigation

Navigation components help users move between major parts of an app or workspace. They are foundational to the experience because they define the user's mental model of the product and what they can access next.

## Example

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_nav(mut commands: Commands) {
    commands.spawn(BeverlyNavigation::new()
        .link("Home")
        .link("Library")
        .link("Reports")
        .link("Settings"));
}
```

## Patterns

- side navigation for dense app shells
- top navigation for lighter content layouts
- breadcrumb trails for hierarchical drill-downs
- tab-like navigation for local or contextual sections

Navigation should be stable, scannable, and reflect the current page or context clearly. Reordering or renaming items without a coherent hierarchy tends to create confusion and higher task abandonment.

## Accessibility

The active item should be obvious, and keyboard order should match the user-facing structure. Clear labels and meaningful grouping are especially important in more complex navigation systems.
