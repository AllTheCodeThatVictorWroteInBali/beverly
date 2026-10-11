//! Radio demo: choose the snack desk's next extremely important task.
//! Run with `-- dark` to start in dark mode; the top-right button toggles light/dark.

use beverly::components::radio::{RadioChanged, RadioGroupBuilder, spawn_radio_group};
use beverly::components::text::{TextRole, ThemedText};
use beverly::components::title::{ThemedTitle, TitleLevel};
use beverly::prelude::*;
use bevy::prelude::*;

#[derive(Component)]
struct SelectionReadout;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BeverlyPlugin)
        .insert_resource(ThemeResource {
            current: theme_from_cli_args(),
        })
        .add_systems(Startup, setup)
        .add_systems(Update, update_selection_readout)
        .run();
}

fn setup(mut commands: Commands) {
    spawn_themed_page(&mut commands, |root| {
        root.spawn(Node {
            width: Val::Px(480.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            row_gap: Val::Px(14.0),
            ..default()
        })
        .with_children(|content| {
            content.spawn((
                ThemedTitle::new(TitleLevel::H2),
                Text::new("Choose today's tiny mission"),
            ));
            content.spawn((
                ThemedText::new(TextRole::Muted),
                Text::new("One choice only. The biscuits have requested governance."),
            ));
            spawn_radio_group(
                content,
                RadioGroupBuilder::new("snack-desk")
                    .options([
                        ("tea", "Prepare a very serious cup of tea"),
                        ("biscuits", "Audit the emergency biscuit supply"),
                        ("fern", "Compliment the office fern"),
                        ("crumb", "Investigate the suspicious crumb"),
                        ("nap", "Schedule a strategic snack nap"),
                    ])
                    .selected("biscuits")
                    .width(Val::Px(460.0))
                    .spacing(Val::Px(6.0)),
            );
            content.spawn((
                SelectionReadout,
                ThemedText::new(TextRole::Body),
                Text::new("Selected: audit the emergency biscuit supply"),
                Node {
                    margin: UiRect::top(Val::Px(8.0)),
                    ..default()
                },
            ));
        });
    });
}

fn update_selection_readout(
    mut changes: MessageReader<RadioChanged>,
    mut readouts: Query<&mut Text, With<SelectionReadout>>,
) {
    for change in changes.read() {
        let label = match change.value.as_str() {
            "tea" => "prepare a very serious cup of tea",
            "biscuits" => "audit the emergency biscuit supply",
            "fern" => "compliment the office fern",
            "crumb" => "investigate the suspicious crumb",
            "nap" => "schedule a strategic snack nap",
            _ => "consider the unknown snack",
        };
        for mut text in &mut readouts {
            *text = Text::new(format!("Selected: {label}"));
        }
    }
}
