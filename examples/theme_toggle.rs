//! Theme toggle demo: the snack desk's day and night shifts.
//! Run with `-- dark` to start in dark mode.

use beverly::components::button::{BeverlyButton, ButtonChild};
use beverly::components::text::{TextRole, ThemedText};
use beverly::components::theme_toggle::toggle_theme;
use beverly::components::title::{ThemedTitle, TitleLevel};
use beverly::prelude::*;
use bevy::prelude::*;

#[derive(Component)]
struct ShiftLabel;

#[derive(Component, Clone, Copy)]
enum Swatch {
    Background,
    Surface,
    Text,
    Border,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BeverlyPlugin)
        .insert_resource(ThemeResource {
            current: theme_from_cli_args(),
        })
        .add_systems(Startup, setup)
        .add_systems(Update, update_shift)
        .run();
}

fn setup(mut commands: Commands) {
    spawn_themed_page(&mut commands, |root| {
        root.spawn(Node {
            width: Val::Px(480.0),
            max_width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(22.0),
            ..default()
        })
        .with_children(|content| {
            content.spawn((
                ThemedTitle::new(TitleLevel::H2),
                Text::new("Snack Desk, Two Shifts"),
            ));
            content.spawn((
                ThemedText::new(TextRole::Muted),
                Text::new("Same biscuits. Different lighting."),
            ));
            content.spawn(
                BeverlyButton::standard("Switch shift")
                    .children([
                        ButtonChild::icon("theme-toggle"),
                        ButtonChild::text("Switch shift"),
                    ])
                    .on("click", toggle_theme),
            );
            content.spawn((ShiftLabel, ThemedText::new(TextRole::Body), Text::new("")));
            content
                .spawn(Node {
                    column_gap: Val::Px(24.0),
                    margin: UiRect::top(Val::Px(12.0)),
                    ..default()
                })
                .with_children(|palette| {
                    for (swatch, label) in [
                        (Swatch::Background, "Page"),
                        (Swatch::Surface, "Surface"),
                        (Swatch::Text, "Text"),
                        (Swatch::Border, "Border"),
                    ] {
                        palette
                            .spawn(Node {
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                row_gap: Val::Px(8.0),
                                ..default()
                            })
                            .with_children(|sample| {
                                sample.spawn((
                                    swatch,
                                    Node {
                                        width: Val::Px(48.0),
                                        height: Val::Px(48.0),
                                        border: UiRect::all(Val::Px(1.0)),
                                        border_radius: BorderRadius::all(Val::Px(6.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::NONE),
                                    BorderColor::all(Color::NONE),
                                ));
                                sample
                                    .spawn((ThemedText::new(TextRole::Caption), Text::new(label)));
                            });
                    }
                });
        });
    });
}

fn shift_label(mode: ThemeMode) -> &'static str {
    match mode {
        ThemeMode::Light => "Day shift: tea, toast, and respectable productivity.",
        ThemeMode::Dark => "Night shift: midnight biscuits, strictly confidential.",
    }
}

fn update_shift(
    theme: Res<ThemeResource>,
    mut labels: Query<&mut Text, With<ShiftLabel>>,
    mut swatches: Query<(&Swatch, &mut BackgroundColor, &mut BorderColor)>,
) {
    for mut text in &mut labels {
        *text = Text::new(shift_label(theme.current.mode));
    }
    let colors = theme.current.colors;
    for (swatch, mut background, mut border) in &mut swatches {
        background.0 = match swatch {
            Swatch::Background => colors.background,
            Swatch::Surface => colors.surface,
            Swatch::Text => colors.text,
            Swatch::Border => colors.border,
        };
        *border = BorderColor::all(colors.border);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_toggle_switches_the_shift_and_palette() {
        let mut app = App::new();
        app.insert_resource(ThemeResource {
            current: light_theme(),
        })
        .add_systems(Update, update_shift);
        let label = app.world_mut().spawn((ShiftLabel, Text::new(""))).id();
        let swatch = app
            .world_mut()
            .spawn((
                Swatch::Background,
                BackgroundColor(Color::NONE),
                BorderColor::all(Color::NONE),
            ))
            .id();
        for mode in [ThemeMode::Light, ThemeMode::Dark, ThemeMode::Light] {
            app.update();
            assert_eq!(app.world().resource::<ThemeResource>().current.mode, mode);
            assert_eq!(app.world().get::<Text>(label).unwrap().0, shift_label(mode));
            assert_eq!(
                app.world().get::<BackgroundColor>(swatch).unwrap().0,
                app.world()
                    .resource::<ThemeResource>()
                    .current
                    .colors
                    .background
            );
            toggle_theme(&mut app.world_mut().commands(), label);
            app.world_mut().flush();
        }
    }
}
