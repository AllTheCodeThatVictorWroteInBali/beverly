//! Sidebar demo: a collapsible snack-desk navigation rail in light/dark mode.
//! Run with `-- dark` to start in dark mode; the top-right button toggles light/dark.

use beverly::components::sidebar::spawn_sidebar;
use beverly::components::text::{TextRole, ThemedText};
use beverly::components::title::{ThemedTitle, TitleLevel};
use beverly::prelude::*;
use beverly::primitives::root::UiFonts;
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
    let ui_fonts = UiFonts {
        text: asset_server.load("embedded://beverly/components/text/fonts/DefaultSans.ttf"),
    };

    spawn_themed_page(&mut commands, move |root| {
        root.spawn(Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Stretch,
            ..default()
        })
        .with_children(|shell| {
            spawn_sidebar(shell, &ui_fonts, 0.0);
            shell
                .spawn(Node {
                    width: percent(70),
                    height: percent(100),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(12.0),
                    padding: UiRect::all(Val::Px(28.0)),
                    ..default()
                })
                .with_children(|content| {
                    content.spawn((ThemedTitle::new(TitleLevel::H2), Text::new("Snack Desk HQ")));
                    content.spawn((
                        ThemedText::new(TextRole::Body),
                        Text::new("Choose a department. The fern is interim manager."),
                    ));
                });
        });
    });
}
