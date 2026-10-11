//! Tabs demo: three departments of the extremely official snack desk.
//! Run with `-- dark` to start in dark mode.

use beverly::components::tabs::{Tab, TabsConfig, spawn_tabs_in_with_bar};
use beverly::components::text::{TextRole, ThemedText};
use beverly::components::title::{ThemedTitle, TitleLevel};
use beverly::icons::IconCommands;
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
            width: Val::Px(620.0),
            max_width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(18.0),
            ..default()
        })
        .with_children(|content| {
            content.spawn((
                ThemedTitle::new(TitleLevel::H2),
                Text::new("Snack Desk Departments"),
            ));
            content.spawn((
                ThemedText::new(TextRole::Muted),
                Text::new("Three teams. One break room. Absolutely no small responsibilities."),
            ));
            content
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(320.0),
                    ..default()
                })
                .with_children(build_departments);
        });
    });
}

fn build_departments(parent: &mut ChildSpawnerCommands) {
    spawn_tabs_in_with_bar(
        parent,
        TabsConfig::new(vec![
            Tab::new("biscuits", "Biscuits"),
            Tab::new("tea", "Tea"),
            Tab::new("toast", "Toast"),
        ]),
        |bar| {
            bar.spawn((ThemedText::new(TextRole::Muted), Text::new("3 on duty")));
        },
        |zone, index, _tab| {
            let (icon, title, status, tasks) = match index {
                0 => (
                    "archive",
                    "Biscuit Bureau",
                    "Inventory: 24 biscuits. Morale: suspiciously high.",
                    [
                        "Count the emergency reserves",
                        "Approve a second biscuit",
                        "Investigate the empty tin",
                    ],
                ),
                1 => (
                    "coffee",
                    "Kettle Committee",
                    "The water is ready. The committee is still deliberating.",
                    [
                        "Select a very serious tea",
                        "Allow three minutes of reflection",
                        "Submit the milk proposal",
                    ],
                ),
                _ => (
                    "sun",
                    "Toast Operations",
                    "Today's forecast: golden, with a chance of butter.",
                    [
                        "Set browning to respectably golden",
                        "Reserve a slice for emergencies",
                        "Keep the jam situation contained",
                    ],
                ),
            };
            zone.spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(22.0)),
                row_gap: Val::Px(16.0),
                ..default()
            })
            .with_children(|panel| {
                panel
                    .spawn(Node {
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(12.0),
                        ..default()
                    })
                    .with_children(|heading| {
                        heading.spawn_feather_sized(icon, 28.0);
                        heading.spawn((ThemedTitle::new(TitleLevel::H3), Text::new(title)));
                    });
                panel.spawn((ThemedText::new(TextRole::Muted), Text::new(status)));
                for (index, task) in tasks.into_iter().enumerate() {
                    panel.spawn((
                        ThemedText::new(TextRole::Body),
                        Text::new(format!("{}. {task}", index + 1)),
                    ));
                }
            });
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use beverly::components::tabs::{TabButton, TabZone, Tabs};

    #[test]
    fn departments_build_three_matching_buttons_and_content_zones() {
        let mut world = World::new();
        world
            .commands()
            .spawn(Node::default())
            .with_children(build_departments);
        world.flush();
        let mut tabs = world.query::<(Entity, &Tabs)>();
        let (entity, tabs) = tabs.single(&world).unwrap();
        assert_eq!(tabs.config.active_tab().unwrap().id, "biscuits");
        let mut buttons = world.query::<&TabButton>();
        let mut zones = world.query::<&TabZone>();
        assert_eq!(buttons.iter(&world).count(), 3);
        assert_eq!(zones.iter(&world).count(), 3);
        for index in 0..3 {
            assert!(
                buttons
                    .iter(&world)
                    .any(|button| button.tabs_entity == entity && button.index == index)
            );
            assert!(
                zones
                    .iter(&world)
                    .any(|zone| zone.tabs_entity == entity && zone.index == index)
            );
        }
    }
}
