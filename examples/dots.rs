//! Dots demo: animated trailing dots that follow light/dark mode.
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
        // Fixed width keeps "Thinking" from shifting as the dots come and go.
        root.spawn(Node {
            width: Val::Px(160.0),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                ThemedText::new(TextRole::Body).size(24.0),
                Text::new("Thinking"),
            ));
            row.spawn((ThemedText::new(TextRole::Body).size(24.0), Dots::new()));
        });
    });
}
