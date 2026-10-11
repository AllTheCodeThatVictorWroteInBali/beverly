//! Search demo: search the snack desk's highly questionable knowledge base.
//! Run with `-- dark` to start in dark mode; the top-right button toggles light/dark.

use beverly::components::input::TextInputEvent;
use beverly::components::search::{Search, spawn_search};
use beverly::components::text::{TextRole, ThemedText};
use beverly::components::title::{ThemedTitle, TitleLevel};
use beverly::prelude::*;
use beverly::theme::ThemeResource;
use bevy::prelude::*;

#[derive(Component)]
struct SearchResults;

const SNACK_INDEX: [&str; 6] = [
    "Emergency biscuit inventory",
    "The office fern's hydration schedule",
    "Missing spoon incident report",
    "Kettle performance review",
    "Strategic snack nap guidelines",
    "Crumb containment procedures",
];

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BeverlyPlugin)
        .insert_resource(ThemeResource {
            current: theme_from_cli_args(),
        })
        .add_systems(Startup, setup)
        .add_systems(Update, sync_search_query)
        .run();
}

fn setup(mut commands: Commands, theme: Res<ThemeResource>) {
    let theme = *theme;
    spawn_themed_page(&mut commands, move |root| {
        root.spawn(Node {
            width: Val::Px(560.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            row_gap: Val::Px(16.0),
            ..default()
        })
        .with_children(|content| {
            content.spawn((
                ThemedTitle::new(TitleLevel::H2),
                Text::new("Search the snack desk"),
            ));
            content.spawn((
                ThemedText::new(TextRole::Muted),
                Text::new("Six documents. Several crumbs. One missing spoon."),
            ));
            spawn_search(content, Handle::<Font>::default(), &theme);
            content.spawn((
                SearchResults,
                ThemedText::new(TextRole::Body),
                Text::new("Type to search the extremely official index."),
                Node {
                    min_height: Val::Px(96.0),
                    ..default()
                },
            ));
        });
    });
}

fn sync_search_query(
    mut input_events: MessageReader<TextInputEvent>,
    input_parents: Query<&ChildOf, With<TextInput>>,
    mut searches: Query<&mut Search>,
    mut results: Query<&mut Text, With<SearchResults>>,
) {
    for event in input_events.read() {
        let (input_entity, query) = match event {
            TextInputEvent::Changed { entity, value }
            | TextInputEvent::Submitted { entity, value } => (*entity, value.as_str()),
            _ => continue,
        };
        let Ok(parent) = input_parents.get(input_entity) else {
            continue;
        };
        let Ok(mut search) = searches.get_mut(parent.parent()) else {
            continue;
        };
        search.query = query.to_string();

        let matches: Vec<_> = SNACK_INDEX
            .iter()
            .filter(|entry| entry.to_lowercase().contains(&query.to_lowercase()))
            .collect();
        let summary = if query.trim().is_empty() {
            "Type to search the extremely official index.".to_string()
        } else if matches.is_empty() {
            format!("No snack records found for '{query}'. The crumbs deny everything.")
        } else {
            format!(
                "{} result{}:\n{}",
                matches.len(),
                if matches.len() == 1 { "" } else { "s" },
                matches
                    .iter()
                    .map(|entry| format!("- {entry}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        };
        for mut text in &mut results {
            *text = Text::new(summary.clone());
        }
    }
}
