# Icons

Beverly includes the fabulous [Feather icons](https://feathericons.com/): a clean, open-source set of outline icons designed by [Cole Bemis and the Feather contributors](https://github.com/feathericons/feather). Feather is distributed under the [MIT license](https://github.com/feathericons/feather/blob/main/LICENSE).

The bundled icons are **SVG vector assets**, not bitmap images or an icon font. They scale cleanly and can be recolored and animated. Beverly embeds them at compile time, so applications do not need to copy an icon assets folder.

## Basic Usage

`BeverlyPlugin` installs the icon system. Import `IconCommands` to spawn icons inside a UI parent:

```rust
use beverly::icons::IconCommands;
use bevy::prelude::*;

fn add_icons(parent: &mut ChildSpawnerCommands) {
    parent.spawn_feather("coffee");
    parent.spawn_feather_sized("heart", 32.0);
    parent.spawn_feather_muted("info", 20.0);
    parent.spawn_feather_colored("check", 24.0, Color::srgb(0.2, 0.7, 0.4));
}
```

Names match the bundled SVG filenames without `.svg`, such as `arrow-left`, `check-circle`, and `alert-triangle`. The default size is 24 pixels.

The regular and sized helpers follow the theme's text color; muted helpers follow its muted text color. Explicitly colored icons retain their supplied color when the theme changes.

## Light/Dark Circle

The additional `theme-toggle` SVG is Beverly's **shadcn/ui-style light/dark circle icon**, used by the page's theme-switch button. It is bundled alongside Feather icons but is not part of the original Feather set. [shadcn/ui](https://ui.shadcn.com/) is the styling reference, not a separate SVG icon pack.

```rust
use beverly::icons::IconCommands;
use bevy::prelude::*;

fn add_theme_icon(parent: &mut ChildSpawnerCommands) {
    parent.spawn_feather_sized("theme-toggle", 24.0);
}
```

This spawns the visual icon only. To make it switch the theme, use the [Theme Toggle](../actions/theme-toggle.md) component or wire `toggle_theme` to your own button.

## Button Icons

`ButtonChild::icon` uses the same embedded icon names:

```rust
use beverly::components::button::{BeverlyButton, ButtonChild};
use bevy::prelude::*;

fn add_save_button(parent: &mut ChildSpawnerCommands) {
    parent.spawn(
        BeverlyButton::standard("Save")
            .children([
                ButtonChild::icon("save"),
                ButtonChild::text("Save"),
            ]),
    );
}
```

Give icon-only controls an accessible name through the owning control's semantics. A decorative icon does not replace a button's accessible label.

## Animation

SVG icons are animatable through Beverly's Bevy components. For example, the `loader` icon automatically spins, and its decorative rotation stops when reduced motion is enabled:

```rust
use beverly::icons::IconCommands;
use bevy::prelude::*;

fn add_loading_icon(parent: &mut ChildSpawnerCommands) {
    parent.spawn_feather_sized("loader", 24.0);
}
```

For a custom size animation, update `IconNode::size` and the UI slot's dimensions together:

```rust
use beverly::icons::{IconCommands, IconNode};
use beverly::theme::AccessibilityVisualPolicyResource;
use bevy::prelude::*;

#[derive(Component)]
struct PulsingIcon;

fn add_pulsing_icon(parent: &mut ChildSpawnerCommands) {
    let icon = parent.spawn_feather_sized("heart", 24.0);
    parent.commands().entity(icon).insert(PulsingIcon);
}

fn pulse_icons(
    time: Res<Time>,
    policy: Res<AccessibilityVisualPolicyResource>,
    mut icons: Query<(&mut IconNode, &mut Node), With<PulsingIcon>>,
) {
    let size = if policy.current.reduced_motion {
        24.0
    } else {
        24.0 + 2.0 * time.elapsed_secs().sin()
    };
    for (mut icon, mut node) in &mut icons {
        icon.size = size;
        node.width = Val::Px(size);
        node.height = Val::Px(size);
    }
}
```

Register `pulse_icons` in `Update`. Reserve a stable outer slot if the changing size should not move neighboring content. SVG rendering is handled by `bevy_svg`; arbitrary SVG/CSS animation declarations are not executed by the icon system.

## Gallery

The icon gallery includes every bundled icon, including the light/dark circle, with names and the shared theme toggle:

```sh
cargo run --example icons
cargo run --example icons -- dark
```

Browse the original [Feather catalog](https://feathericons.com/) for designs and names, and the [Feather repository](https://github.com/feathericons/feather) for upstream SVGs and attribution.