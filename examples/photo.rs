//! Photo demo: a rounded portrait that follows the shared light/dark page theme.
//! Run with `-- dark` to start in dark mode; the top-right button toggles light/dark.

use beverly::components::photo::{Photo, spawn_photo};
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

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let portrait = asset_server.load("avatars/profile.png");
    spawn_themed_page(&mut commands, move |root| {
        root.spawn(Node {
            width: Val::Px(360.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(14.0),
            ..default()
        })
        .with_children(|content| {
            content.spawn((
                ThemedTitle::new(TitleLevel::H3),
                Text::new("Meet the snack inspector"),
            ));
            spawn_photo(
                content,
                Photo::new(portrait).size(320.0, 320.0).border_radius(22.0),
            );
            content.spawn((
                ThemedText::new(TextRole::Muted),
                Text::new("A very serious professional. The crumbs have been warned."),
            ));
        });
    });
}
