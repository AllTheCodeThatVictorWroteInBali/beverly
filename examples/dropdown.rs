//! Dropdown demo: one dropdown that follows light/dark mode.
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
        // The dropdown stretches to its parent's width.
        root.spawn(Node {
            width: Val::Percent(100.0),
            max_width: Val::Px(280.0),
            min_width: Val::Px(0.0),
            flex_shrink: 0.0,
            ..default()
        })
        .with_children(|slot| {
            spawn_dropdown(
                slot,
                DropdownConfig::new("Select a fruit")
                    .option("Apple", "apple")
                    .option("Banana", "banana")
                    .option("Blueberry", "blueberry")
                    .option("Grapes", "grapes")
                    .option("Pineapple", "pineapple"),
            );
        });
    });
}
