//! Minimal example demonstrating several Beverly components together.

use beverly::components::checkbox::CheckboxState;
use beverly::components::toggle::{ToggleConfig, ToggleShadowMaterial, spawn_toggle};
use beverly::prelude::*;
use bevy::prelude::*;

#[path = "support/mod.rs"]
mod demo_support;
use demo_support::DemoAppExt;

#[derive(Component)]
enum ProfileField {
    Name,
    Email,
}

#[derive(Component)]
struct Newsletter;

#[derive(Component)]
struct Notifications;

#[derive(Component)]
struct SaveProfile;

#[derive(Component)]
struct SaveStatus;

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BeverlyPlugin)
        .insert_resource(ThemeResource {
            current: theme_from_cli_args(),
        })
        .add_systems(Startup, setup)
        .add_systems(Update, save_profile)
        .run_demo();
}

fn setup(
    mut commands: Commands,
    theme: Res<ThemeResource>,
    mut materials: ResMut<Assets<ToggleShadowMaterial>>,
) {
    spawn_themed_page(&mut commands, |root| {
        root.spawn(Node {
            width: Val::Percent(100.0),
            max_width: Val::Px(760.0),
            min_width: Val::Px(0.0),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(16.0),
            ..default()
        })
        .with_children(|content| {
            content.spawn((
                ThemedTitle::new(TitleLevel::H2),
                Text::new("Profile settings"),
            ));
            for (field, label, value, kind) in [
                (
                    ProfileField::Name,
                    "Display name",
                    "Alex Morgan",
                    TextInputKind::Text,
                ),
                (
                    ProfileField::Email,
                    "Email address",
                    "alex@example.com",
                    TextInputKind::Email,
                ),
            ] {
                content
                    .spawn(Node {
                        min_width: Val::Px(0.0),
                        flex_shrink: 0.0,
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(6.0),
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn((ThemedText::new(TextRole::Label), Text::new(label)));
                        let input = spawn_text_input(
                            row,
                            TextInputConfig::new(label)
                                .label(label)
                                .initial_value(value)
                                .kind(kind),
                        );
                        row.commands().entity(input).insert(field);
                    });
            }
            let checkbox = spawn_checkbox(
                content,
                CheckboxConfig::new()
                    .label("Receive the monthly newsletter")
                    .checked(true),
            );
            content.commands().entity(checkbox).insert(Newsletter);
            let toggle = spawn_toggle(
                content,
                ToggleConfig::new()
                    .label("Desktop notifications")
                    .checked(true),
                &theme,
                &mut materials,
            );
            content.commands().entity(toggle).insert(Notifications);
            content
                .spawn(Node {
                    min_width: Val::Px(0.0),
                    flex_shrink: 0.0,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(8.0),
                    row_gap: Val::Px(8.0),
                    ..default()
                })
                .with_children(|actions| {
                    actions.spawn((SaveProfile, BeverlyButton::primary("Save profile")));
                });
            content.spawn((
                SaveStatus,
                ThemedText::new(TextRole::Muted),
                Text::new("Not saved"),
            ));
        });
    });
}

fn save_profile(
    buttons: Query<&Interaction, (Changed<Interaction>, With<SaveProfile>)>,
    fields: Query<(&ProfileField, &TextInput)>,
    newsletters: Query<&CheckboxState, With<Newsletter>>,
    notifications: Query<&Toggle, With<Notifications>>,
    mut statuses: Query<&mut Text, With<SaveStatus>>,
) {
    if !buttons
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed)
    {
        return;
    }
    let mut name = "";
    let mut email = "";
    for (field, input) in &fields {
        match field {
            ProfileField::Name => name = input.value.trim(),
            ProfileField::Email => email = input.value.trim(),
        }
    }
    let message = if name.is_empty() || email.is_empty() {
        "Enter a display name and email address before saving.".to_owned()
    } else {
        let newsletter = newsletters.iter().any(|state| state.checked);
        let notifications = notifications.iter().any(|state| state.checked);
        format!(
            "Saved locally: {name} ({email}). Newsletter: {}. Notifications: {}.",
            if newsletter { "on" } else { "off" },
            if notifications { "on" } else { "off" }
        )
    };
    for mut status in &mut statuses {
        status.0 = message.clone();
    }
}
