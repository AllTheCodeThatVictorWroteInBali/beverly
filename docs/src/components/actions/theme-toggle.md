# Theme Toggle

The theme toggle switches between Beverly's light and dark themes. The active theme lives in `ThemeResource`; themed text, icons, and component surfaces follow that resource.

`BeverlyPlugin` installs `ThemeTogglePlugin` alongside the button, icon, and theme systems.

## Themed Page

`spawn_themed_page` creates a full-window page with centered column content and an icon-only toggle in the top-right corner.

```rust
use beverly::components::text::{TextRole, ThemedText};
use beverly::prelude::*;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BeverlyPlugin)
        .insert_resource(ThemeResource {
            current: theme_from_cli_args(),
        })
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    spawn_themed_page(&mut commands, |content| {
        content.spawn((
            ThemedText::new(TextRole::Heading),
            Text::new("Same biscuits. Different lighting."),
        ));
    });
}
```

`theme_from_cli_args()` starts in light mode unless the process has an argument exactly equal to `dark`.

## Standalone Toggle

Use `spawn_theme_toggle` within an existing UI parent:

```rust
use beverly::components::theme_toggle::spawn_theme_toggle;
use bevy::prelude::*;

fn add_toggle(parent: &mut ChildSpawnerCommands) {
    spawn_theme_toggle(parent);
}
```

The helper creates an absolutely positioned wrapper, inset 16 pixels from the parent's top and right edges. It returns the **wrapper entity**, not the inner button. The button uses the embedded `theme-toggle` icon and Beverly's standard small button style.

This helper does not create a page or synchronize arbitrary background colors. `ThemeTogglePlugin` updates the background of entities marked `ThemedPage` and the window's `ClearColor` when the theme changes. Other surfaces should use theme-aware components or read `ThemeResource` themselves.

## Custom Button

Reuse `toggle_theme` when the control belongs in your own toolbar or content layout:

```rust
use beverly::components::button::{BeverlyButton, ButtonChild};
use beverly::components::theme_toggle::toggle_theme;
use bevy::prelude::*;

fn add_shift_button(parent: &mut ChildSpawnerCommands) {
    parent.spawn(
        BeverlyButton::standard("Switch shift")
            .children([
                ButtonChild::icon("theme-toggle"),
                ButtonChild::text("Switch shift"),
            ])
            .on("click", toggle_theme),
    );
}
```

The callback queues a change to `ThemeResource`. Light mode becomes `dark_theme()`; dark mode becomes `light_theme()`. Both controls on the same page share this state.

## Theme State

- The stock callback replaces the entire theme with a built-in preset. Custom colors, typography, spacing, or motion tokens are not preserved. Applications with custom light/dark palettes should provide their own callback.
- The initial theme comes from application configuration or `theme_from_cli_args()`, not the operating system's appearance setting.
- The toggle does not persist a preference between application launches. Store the chosen mode in application settings if persistence is needed.
- Only theme-aware UI updates automatically. Explicit color overrides remain application-owned.

## Example

Run the dedicated demo from the repository root:

```sh
cargo run --example theme_toggle
cargo run --example theme_toggle -- dark
```

The demo includes a custom switch button, a live day/night shift label, and swatches for the page, surface, text, and border colors.