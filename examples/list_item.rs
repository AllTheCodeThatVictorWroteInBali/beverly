//! List item demo: five playful rows that follow light/dark mode.
//! Run with `-- dark` to start in dark mode; the top-right button toggles light/dark.

use beverly::components::list_item::spawn_list_item;
use beverly::icons::Icon;
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
            width: Val::Px(520.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            row_gap: Val::Px(10.0),
            ..default()
        })
        .with_children(|list| {
            spawn_list_item(
                list,
                "Meeting with the office fern",
                Some("It requested more sunlight and fewer spreadsheets.".to_string()),
                Some(Icon::feather("sun")),
            );
            spawn_list_item(
                list,
                "Emergency snack audit",
                Some("Two cookies accounted for. One crumb remains at large.".to_string()),
                Some(Icon::feather("coffee")),
            );
            spawn_list_item(
                list,
                "Inbox hydra update",
                Some("You cleared one email. Three have taken its place.".to_string()),
                Some(Icon::feather("mail")),
            );
            spawn_list_item(
                list,
                "Water the dramatic cactus",
                Some("Thriving, somehow. Still judging your choices.".to_string()),
                Some(Icon::feather("droplet")),
            );
            spawn_list_item(
                list,
                "Cloud status report",
                Some("Mostly fluffy, with a chance of tiny hats.".to_string()),
                Some(Icon::feather("cloud")),
            );
        });
    });
}
