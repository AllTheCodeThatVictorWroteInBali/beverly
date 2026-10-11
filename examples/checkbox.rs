//! Checkbox demo: one checkbox that follows light/dark mode.
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
        // The checkbox stretches to its parent's width, so give it a fixed-width one.
        root.spawn(Node {
            width: Val::Px(280.0),
            ..default()
        })
        .with_children(|slot| {
            spawn_checkbox(
                slot,
                CheckboxConfig::new()
                    .label("I'm fun to click")
                    .checked(true),
            );
        });
    });
}
