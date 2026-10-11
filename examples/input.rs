//! Input demo: a text input that follows light/dark mode.
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
        // The input stretches to its parent's width, so give it a fixed-width one.
        root.spawn(Node {
            width: Val::Px(320.0),
            ..default()
        })
        .with_children(|slot| {
            spawn_text_input(
                slot,
                TextInputConfig::new("Letters only...")
                    .label("Your name")
                    .char_filter(|ch| !ch.is_ascii_digit())
                    .error_message("Numbers aren't on the guest list. Letters only, please!"),
            );
        });
    });
}
