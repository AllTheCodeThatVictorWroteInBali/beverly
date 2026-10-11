//! Divider demo: a horizontal and a vertical divider that follow light/dark mode.
//! Run with `-- dark` to start in dark mode; the top-right button toggles light/dark.

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
            width: Val::Px(320.0),
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
