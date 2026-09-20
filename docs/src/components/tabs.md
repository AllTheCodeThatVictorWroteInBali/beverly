# Tabs

Tabs organize several related views within the same area. They are useful for settings, dashboards, or content sections that share a common context and need to switch without leaving the parent surface.

## Example

```rust
use bevy::prelude::*;
use beverly::prelude::*;

fn build_tabs(mut commands: Commands) {
    commands.spawn(BeverlyTabs::new()
        .tab("Overview")
        .tab("Activity")
        .tab("Settings"));
}
```

## Guidance

- use tabs for local navigation, not global app navigation
- keep the number of tabs manageable
- ensure the active tab is visually and semantically clear
- support arrow-key navigation when possible
- keep the content behind the tabs consistent with the selected panel

## Accessibility

Tabs should provide a visible selected state, a clear focus order, and labels that make sense independently. Users should be able to understand which panel is active without relying only on color or hover styling.
