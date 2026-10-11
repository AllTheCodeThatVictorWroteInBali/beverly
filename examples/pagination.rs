//! Pagination demo: eight tiny reports from the office's extremely serious snack desk.
//! Run with `-- dark` to start in dark mode; the top-right button toggles light/dark.

use beverly::components::pagination::{PaginationConfig, PaginationEvent, spawn_pagination};
use beverly::components::text::{TextRole, ThemedText};
use beverly::components::title::{ThemedTitle, TitleLevel};
use beverly::prelude::*;
use bevy::prelude::*;

#[derive(Component)]
struct PageCopy;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BeverlyPlugin)
        .insert_resource(ThemeResource {
            current: theme_from_cli_args(),
        })
        .add_systems(Startup, setup)
        .add_systems(Update, update_page_copy)
        .run();
}

fn setup(mut commands: Commands) {
    spawn_themed_page(&mut commands, |root| {
        root.spawn(Node {
            width: Val::Px(760.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            row_gap: Val::Px(22.0),
            ..default()
        })
        .with_children(|content| {
            content.spawn((
                ThemedTitle::new(TitleLevel::H2),
                Text::new("The Extremely Important Snack Desk"),
            ));
            content.spawn((
                PageCopy,
                ThemedText::new(TextRole::Body),
                Text::new(page_copy(1)),
                TextFont {
                    font_size: FontSize::Px(20.0),
                    ..default()
                },
                Node {
                    min_height: Val::Px(100.0),
                    ..default()
                },
            ));
            spawn_pagination(
                content,
                PaginationConfig {
                    page_size: 3,
                    total_pages: 8,
                    show_first_last: true,
                    show_page_numbers: true,
                    max_page_buttons: 5,
                },
            );
        });
    });
}

fn update_page_copy(
    mut events: MessageReader<PaginationEvent>,
    mut copy: Query<&mut Text, With<PageCopy>>,
) {
    for event in events.read() {
        for mut text in &mut copy {
            *text = Text::new(page_copy(event.page));
        }
    }
}

fn page_copy(page: usize) -> String {
    match page {
        1 => "Report 1: The biscuits have formed a surprisingly effective union.".to_string(),
        2 => "Report 2: A tiny spoon has gone missing. The yogurt is under investigation."
            .to_string(),
        3 => "Report 3: The office fern requests sunlight and one (1) respectful compliment."
            .to_string(),
        4 => "Report 4: Crumb levels are elevated. Morale is somehow even higher.".to_string(),
        5 => "Report 5: The emergency chocolate is no longer considered an emergency.".to_string(),
        6 => "Report 6: A grape has rolled beneath the fridge and started a new life.".to_string(),
        7 => "Report 7: The kettle has completed its annual performance review.".to_string(),
        _ => "Report 8: All snacks accounted for. One crumb remains at large.".to_string(),
    }
}
