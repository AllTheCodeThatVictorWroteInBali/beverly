//! Button demo: small, medium and large in every color. The default button follows the theme mode.
//! Run with `-- dark` to start in dark mode; the top-right button toggles light/dark.

use beverly::prelude::*;
use bevy::prelude::*;

pub fn main() {
    #[cfg(target_arch = "wasm32")]
    let plugins = DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            canvas: Some("#bevy".to_string()),
            fit_canvas_to_parent: true,
            ..default()
        }),
        ..default()
    });
    #[cfg(not(target_arch = "wasm32"))]
    let plugins = DefaultPlugins;

    #[cfg(target_arch = "wasm32")]
    let initial_theme = light_theme();
    #[cfg(not(target_arch = "wasm32"))]
    let initial_theme = theme_from_cli_args();

    App::new()
        .add_plugins(plugins)
        .add_plugins(BeverlyPlugin)
        .insert_resource(ThemeResource {
            current: initial_theme,
        })
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    spawn_themed_page(&mut commands, |root| {
        let rows = [
            (ButtonSize::Sm, false),
            (ButtonSize::Md, false),
            (ButtonSize::Lg, false),
            (ButtonSize::Md, true),
        ];
        for (size, disabled) in rows {
            root.spawn(Node {
                width: Val::Percent(100.0),
                align_items: AlignItems::Center,
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(12.0),
                row_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|row| {
                let buttons = [
                    BeverlyButton::standard("Default"),
                    BeverlyButton::primary("Primary"),
                    BeverlyButton::secondary("Secondary"),
                    BeverlyButton::success("Success"),
                    BeverlyButton::danger("Danger"),
                    BeverlyButton::warning("Warning"),
                    BeverlyButton::info("Info"),
                    BeverlyButton::light("Light"),
                    BeverlyButton::dark("Dark"),
                ];
                for button in buttons {
                    row.spawn(button.sized(size).disabled(disabled));
                }
            });
        }
    });
}
