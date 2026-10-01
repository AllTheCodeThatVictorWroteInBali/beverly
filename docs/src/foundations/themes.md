# Themes

<img src="../assets/beverly_logo_final.png" alt="Beverly brand mark" width="280" />

Themes give an application one shared visual language. Beverly's built-in themes are light and dark; each provides semantic colors and shared design values while leaving component structure and behavior unchanged.

The active theme is a global Bevy resource, not a component attached to a root UI node. Beverly components read from that resource, so changing it updates the theme values they use throughout the interface.

## The Global Theme

Beverly's `BeverlyPlugin` installs `ThemePlugin`, which initializes `ThemeResource` with `light_theme()`. To start in dark mode, insert a resource containing `dark_theme()` after adding `BeverlyPlugin`:

```rust
use bevy::prelude::*;
use beverly::prelude::{BeverlyPlugin, ThemeResource, dark_theme};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BeverlyPlugin)
        .insert_resource(ThemeResource {
            current: dark_theme(),
        })
        .run();
}
```

This is application-wide configuration: cards, buttons, text, form controls, navigation, and other theme-aware Beverly components all use the same active palette. There is no need to set a separate theme on each screen or widget.

## Switching Light And Dark

Send a `ThemeChanged` message to switch modes at runtime. The theme plugin replaces the current theme with the corresponding built-in preset, and components that read `ThemeResource` use the new values:

```rust
use bevy::prelude::*;
use beverly::prelude::ThemeMode;
use beverly::theme::ThemeChanged;

fn switch_to_dark(mut theme_changes: MessageWriter<ThemeChanged>) {
    theme_changes.write(ThemeChanged {
        mode: ThemeMode::Dark,
    });
}
```

`ThemeMode` currently has `Light` and `Dark` variants. High-contrast and reduced-effects policies are configured separately through Beverly's accessibility APIs; they are not additional built-in `ThemeMode` values.

## Why This Feels Cohesive

The presets keep semantic roles stable while changing their values. For example, components use `background`, `surface`, `text`, `text_muted`, `primary`, `border`, and `focus` rather than each choosing unrelated colors. Light and dark themes provide different colors for those same roles, preserving hierarchy and intent as the whole application changes appearance.

`Theme` also groups shared typography sizes, spacing, corner radii, border widths, shadows, visual-effect defaults, and transitions. Components can therefore share more than a palette, and a product can tune its design language centrally instead of accumulating one-off values in each screen.

Use semantic component variants for intent (such as success or danger) and reserve direct token customization for genuine product-specific needs. Avoid hard-coding colors for ordinary component states: local literals can break contrast or make one widget feel disconnected when the global mode changes.

## Customizing The Presets

The current theme can be customized as a resource. This example changes the primary color while retaining the rest of the light preset:

```rust
use bevy::prelude::*;
use beverly::prelude::{ThemeResource, light_theme};

fn set_brand_color(mut theme: ResMut<ThemeResource>) {
    theme.current.colors.primary = Color::srgb(0.12, 0.38, 0.72);
}
```

Apply custom values after a mode change if they should persist: sending `ThemeChanged` selects a fresh built-in light or dark preset. Also note that not every color is mode-dependent. Tokens such as `light_surface` and `dark_surface` intentionally remain fixed because they represent component variants, not the app's current appearance.
