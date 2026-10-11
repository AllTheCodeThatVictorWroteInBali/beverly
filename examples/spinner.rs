//! Spinner demo: the snack desk's very busy waiting queue.
//! Run with `-- dark` to start in dark mode.

use beverly::components::text::{TextRole, ThemedText};
use beverly::components::title::{ThemedTitle, TitleLevel};
use beverly::icons::IconCommands;
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
            width: Val::Px(500.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            row_gap: Val::Px(24.0),
            ..default()
        })
        .with_children(|content| {
            content.spawn((
                ThemedTitle::new(TitleLevel::H2),
                Text::new("Please hold the biscuits"),
            ));
            content.spawn((
                ThemedText::new(TextRole::Muted),
                Text::new("The snack desk is processing several urgent non-emergencies."),
            ));
            loading_row(
                content,
                20.0,
                "Checking the biscuit inventory",
                "One for the desk. One for quality assurance.",
            );
            loading_row(
                content,
                32.0,
                "Negotiating with the kettle",
                "It has requested another moment to reflect.",
            );
            loading_row(
                content,
                48.0,
                "Preparing the grand toast reveal",
                "The suspense is lightly buttered.",
            );
        });
    });
}

fn loading_row(parent: &mut ChildSpawnerCommands, size: f32, label: &str, detail: &str) {
    parent
        .spawn(Node {
            min_height: Val::Px(72.0),
            align_items: AlignItems::Center,
            column_gap: Val::Px(18.0),
            ..default()
        })
        .with_children(|row| {
            row.spawn(Node {
                width: Val::Px(56.0),
                height: Val::Px(56.0),
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            })
            .with_children(|icon| {
                icon.spawn_feather_sized("loader", size);
            });
            row.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                flex_grow: 1.0,
                min_width: Val::Px(0.0),
                ..default()
            })
            .with_children(|text| {
                text.spawn((ThemedText::new(TextRole::Body), Text::new(label)));
                text.spawn((ThemedText::new(TextRole::Muted), Text::new(detail)));
            });
        });
}
