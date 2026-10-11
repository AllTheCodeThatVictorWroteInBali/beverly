//! File input demo: a file picker that follows light/dark mode.
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
        spawn_file_input_in(
            root,
            FileInput::new("Drag and drop an image here").images(),
        );
        root.spawn((
            ThemedText::new(TextRole::Muted),
            Text::new("Images show a preview; other files just load."),
        ));
    });
}
