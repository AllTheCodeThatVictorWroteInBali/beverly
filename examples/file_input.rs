//! File input demo: a file picker that follows light/dark mode.
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
            max_width: Val::Px(360.0),
            min_width: Val::Px(0.0),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(16.0),
            ..default()
        }).with_children(|content| {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let input = spawn_file_input_in(
                    content,
                    FileInput::new("Drag and drop an image here").images(),
                );
                content.commands().entity(input).queue(|mut entity: EntityWorldMut| {
                    if let Some(mut node) = entity.get_mut::<Node>() {
                        node.width = Val::Percent(100.0);
                        node.min_width = Val::Px(0.0);
                        node.flex_shrink = 0.0;
                    }
                });
                content.spawn((
                    ThemedText::new(TextRole::Muted),
                    Text::new("Images show a preview; other files just load."),
                ));
            }
            #[cfg(target_arch = "wasm32")]
            {
                content.spawn(BeverlyButton::standard("File picker unavailable").icon("upload").disabled(true));
                content.spawn((
                    ThemedText::new(TextRole::Muted),
                    Text::new("File selection and image previews are available in the native demo, not this browser embed."),
                ));
            }
        });
    });
}
