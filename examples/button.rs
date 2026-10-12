//! Button demo: small, medium and large in every color. The default button follows the theme mode.
//! Run with `-- dark` to start in dark mode; the top-right button toggles light/dark.

use beverly::prelude::*;
use bevy::prelude::*;

pub fn main() {
    #[cfg(target_arch = "wasm32")]
    let plugins = DefaultPlugins.set(WindowPlugin {
        primary_window: Some(button_demo_window()),
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

#[cfg(target_arch = "wasm32")]
fn button_demo_window() -> Window {
    let viewport = web_sys::window().map(|browser_window| {
        let width = browser_window
            .inner_width()
            .ok()
            .and_then(|value| value.as_f64())
            .unwrap_or(800.0)
            .round()
            .clamp(1.0, u32::MAX as f64) as u32;
        let height = browser_window
            .inner_height()
            .ok()
            .and_then(|value| value.as_f64())
            .unwrap_or(580.0)
            .round()
            .clamp(1.0, u32::MAX as f64) as u32;
        (width, height)
    });
    let (width, height) = viewport.unwrap_or((800, 580));

    Window {
        resolution: bevy::window::WindowResolution::new(width, height),
        canvas: Some("#bevy".to_string()),
        fit_canvas_to_parent: true,
        ..default()
    }
}

fn setup(mut commands: Commands) {
    spawn_themed_page(&mut commands, |root| {
        let rows = [
            ("Small", ButtonSize::Sm, false),
            ("Medium", ButtonSize::Md, false),
            ("Large", ButtonSize::Lg, false),
            ("Disabled", ButtonSize::Md, true),
        ];
        root.spawn(Node {
            width: Val::Percent(100.0),
            max_width: Val::Px(1120.0),
            padding: UiRect::all(Val::Px(16.0)),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(12.0),
            ..default()
        })
        .with_children(|groups| {
            for (label, size, disabled) in rows {
                groups
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(6.0),
                        ..default()
                    })
                    .with_children(|group| {
                        group.spawn((ThemedText::new(TextRole::Label), Text::new(label)));
                        group
                            .spawn(Node {
                                width: Val::Percent(100.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                flex_wrap: FlexWrap::Wrap,
                                column_gap: Val::Px(8.0),
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
                    });
            }
        });
    });
}
