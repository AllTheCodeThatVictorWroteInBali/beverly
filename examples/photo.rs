//! Photo demo: a rounded portrait that follows the shared light/dark page theme.
//! Run with `-- dark` to start in dark mode; the top-right button toggles light/dark.

use beverly::components::photo::{Photo, spawn_photo};
use beverly::components::text::{TextRole, ThemedText};
use beverly::components::title::{ThemedTitle, TitleLevel};
use beverly::prelude::*;
use bevy::prelude::*;

#[path = "support/mod.rs"]
mod demo_support;
use demo_support::DemoAppExt;

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BeverlyPlugin)
        .insert_resource(ThemeResource {
            current: theme_from_cli_args(),
        })
        .add_systems(Startup, setup)
        .run_demo();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let portrait = asset_server.load("avatars/profile.png");
    spawn_themed_page(&mut commands, move |root| {
        root.spawn(Node {
            width: Val::Percent(100.0),
            max_width: Val::Px(360.0),
            min_width: Val::Px(0.0),
            flex_shrink: 0.0,
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
            let photo = spawn_photo(
                content,
                Photo::new(portrait).size(320.0, 320.0).border_radius(22.0),
            );
            content
                .commands()
                .entity(photo)
                .queue(|mut entity: EntityWorldMut| {
                    if let Some(mut node) = entity.get_mut::<Node>() {
                        node.width = Val::Percent(100.0);
                        node.max_width = Val::Px(320.0);
                        node.min_width = Val::Px(0.0);
                        node.height = Val::Auto;
                        node.aspect_ratio = Some(1.0);
                        node.flex_shrink = 0.0;
                    }
                });
            content.spawn((
                ThemedText::new(TextRole::Muted),
                Text::new("A very serious professional. The crumbs have been warned."),
            ));
        });
    });
}
