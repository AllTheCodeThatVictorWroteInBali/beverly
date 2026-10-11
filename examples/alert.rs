//! Alert demo: every variant. A plain alert defaults to white in light mode, dark in dark mode.
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
            width: Val::Px(384.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            row_gap: Val::Px(12.0),
            ..default()
        })
        .with_children(|root| {
            root.spawn(Alert::primary("This is a primary alert—check it out!"));
            root.spawn(Alert::secondary("This is a secondary alert—check it out!"));
            root.spawn(Alert::success("This is a success alert—check it out!"));
            root.spawn(Alert::danger("This is a danger alert—check it out!"));
            root.spawn(Alert::warning("This is a warning alert—check it out!"));
            root.spawn(Alert::info("This is a info alert—check it out!"));
            root.spawn(Alert::light("This is a light alert—check it out!"));
            root.spawn(Alert::dark("This is a dark alert—check it out!"));
        });
    });
}
