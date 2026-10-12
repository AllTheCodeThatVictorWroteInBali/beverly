# Toggle Switch

A toggle switch represents an on/off setting. `ToggleConfig` supplies the label,
initial checked state, and disabled state; the spawned `Toggle` component holds
the current state. `BeverlyPlugin` installs its interaction and visual systems.

## Live Demo

<iframe src="../../assets/demos/index.html?demo=toggle" title="Interactive Beverly toggle switch demo" loading="lazy" allow="fullscreen" style="display: block; width: 100%; height: min(820px, 90vh); min-height: 580px; border: 0;"></iframe>

<a href="../../assets/demos/index.html?demo=toggle" target="_blank" rel="noopener">Open the demo in a separate tab</a>.

## Basic Usage

```rust
use beverly::components::toggle::{ToggleConfig, ToggleShadowMaterial, spawn_toggle};
use beverly::prelude::ThemeResource;
use bevy::prelude::*;

fn add_toggle(
    parent: &mut ChildSpawnerCommands,
    theme: &ThemeResource,
    materials: &mut Assets<ToggleShadowMaterial>,
) -> Entity {
    spawn_toggle(
        parent,
        ToggleConfig::new().label("Notifications").checked(true),
        theme,
        materials,
    )
}
```

Pass the application's `ResMut<Assets<ToggleShadowMaterial>>` into this helper;
do not create a temporary asset collection. Use `.disabled(true)` for a locked
setting, or `.thumb_icons(...)` to configure the two thumb icons.

Read `Toggle::checked` through a query when synchronizing application settings
or status text. The demo counts checked switches, including the disabled one.
This API does not require the planned model-binding DSL.