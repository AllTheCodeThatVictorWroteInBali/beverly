//! Divider demo: a horizontal and a vertical divider that follow light/dark mode.
//! Run with `-- dark` to start in dark mode; the top-right button toggles light/dark.

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

fn setup(mut commands: Commands) {
    spawn_themed_page(&mut commands, |root| {
        root.spawn(Node {
            width: Val::Percent(100.0),
            max_width: Val::Px(320.0),
            min_width: Val::Px(0.0),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|column| {
            column.spawn((ThemedTitle::new(TitleLevel::H4), Text::new("Beverly")));
            column.spawn((
                ThemedText::new(TextRole::Muted),
                Text::new("Native UI, light and dark."),
            ));

            column.spawn(
                Divider::horizontal()
                    .margin(UiRect::vertical(Val::Px(16.0)))
                    .build(),
            );

            column
                .spawn(Node {
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|row| {
                    for (index, label) in ["Docs", "Blog", "Source"].into_iter().enumerate() {
                        if index > 0 {
                            row.spawn(
                                Divider::vertical()
                                    .length(Val::Px(16.0))
                                    .margin(UiRect::horizontal(Val::Px(16.0)))
                                    .build(),
                            );
                        }
                        row.spawn((ThemedText::new(TextRole::Body), Text::new(label)));
                    }
                });
        });
    });
}
