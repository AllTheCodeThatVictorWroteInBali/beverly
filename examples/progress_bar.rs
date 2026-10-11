//! Progress bar demo: four highly important office missions in light/dark mode.
//! Run with `-- dark` to start in dark mode; the top-right button toggles light/dark.

use beverly::components::progress_bar::{ProgressBar, spawn_progress_bar_into};
use beverly::components::text::{TextRole, ThemedText};
use beverly::components::title::{ThemedTitle, TitleLevel};
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
    spawn_themed_page(&mut commands, |root| {
        root.spawn(Node {
            width: Val::Px(560.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            row_gap: Val::Px(22.0),
            ..default()
        })
        .with_children(|content| {
            content.spawn((
                ThemedTitle::new(TitleLevel::H2),
                Text::new("Today's tiny missions"),
            ));
            progress_row(content, "Locate the emergency biscuits", 0.82);
            progress_row(content, "Convince the fern it is doing great", 0.56);
            progress_row(content, "Wait for the kettle to feel ready", 0.34);
            progress_row(content, "File the crumb incident report", 1.0);
            content.spawn((
                ThemedText::new(TextRole::Muted),
                Text::new("The crumb report is complete. The crumb remains at large."),
            ));
        });
    });
}

fn progress_row(parent: &mut ChildSpawnerCommands, label: &str, progress: f32) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                ThemedText::new(TextRole::Body),
                Text::new(format!("{label} · {}%", (progress * 100.0).round() as u32)),
            ));
            spawn_progress_bar_into(row, ProgressBar::new().progress(progress).size(560.0, 12.0));
        });
}
