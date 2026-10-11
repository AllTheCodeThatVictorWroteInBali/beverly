//! Toggle demo: configure the snack desk's highly important safeguards.
//! Run with `-- dark` to start in dark mode.

use beverly::components::text::{TextRole, ThemedText};
use beverly::components::title::{ThemedTitle, TitleLevel};
use beverly::components::toggle::{Toggle, ToggleConfig, ToggleShadowMaterial, spawn_toggle};
use beverly::icons::Icon;
use beverly::prelude::*;
use bevy::prelude::*;

#[derive(Component)]
struct DeskSetting;

#[derive(Component)]
struct SafeguardReadout;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BeverlyPlugin)
        .insert_resource(ThemeResource {
            current: theme_from_cli_args(),
        })
        .add_systems(Startup, setup)
        .add_systems(Update, update_readout)
        .run();
}

fn setup(
    mut commands: Commands,
    theme: Res<ThemeResource>,
    mut materials: ResMut<Assets<ToggleShadowMaterial>>,
) {
    spawn_themed_page(&mut commands, |root| {
        root.spawn(Node {
            width: Val::Px(500.0),
            max_width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(18.0),
            ..default()
        })
        .with_children(|content| {
            content.spawn((
                ThemedTitle::new(TitleLevel::H2),
                Text::new("Snack Desk Safeguards"),
            ));
            content.spawn((
                ThemedText::new(TextRole::Muted),
                Text::new("Small switches. Extremely official consequences."),
            ));
            for (config, detail) in [
                (
                    ToggleConfig::new()
                        .label("Protect the emergency biscuits")
                        .checked(true),
                    "The tin is under round-the-clock snack supervision.",
                ),
                (
                    ToggleConfig::new().label("Send second-breakfast reminders"),
                    "A gentle nudge when the toast feels neglected.",
                ),
                (
                    ToggleConfig::new()
                        .label("Enable quiet kettle hours")
                        .thumb_icons(Icon::feather("sun"), Icon::feather("moon")),
                    "All boiling must be conducted in a respectful manner.",
                ),
                (
                    ToggleConfig::new()
                        .label("Mandatory crumb inspection")
                        .checked(true)
                        .disabled(true),
                    "Locked by the Crumb Committee. No exceptions.",
                ),
            ] {
                content
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(4.0),
                        ..default()
                    })
                    .with_children(|row| {
                        let entity = spawn_toggle(row, config, &theme, &mut materials);
                        row.commands().entity(entity).insert(DeskSetting);
                        row.spawn((ThemedText::new(TextRole::Caption), Text::new(detail)));
                    });
            }
            content.spawn((
                SafeguardReadout,
                ThemedText::new(TextRole::Body),
                Text::new("2 / 4 safeguards active"),
            ));
        });
    });
}

fn update_readout(
    settings: Query<&Toggle, With<DeskSetting>>,
    mut readouts: Query<&mut Text, With<SafeguardReadout>>,
) {
    let total = settings.iter().count();
    let active = settings.iter().filter(|setting| setting.checked).count();
    let label = format!("{active} / {total} safeguards active");
    for mut text in &mut readouts {
        if text.0 != label {
            *text = Text::new(label.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readout_tracks_enabled_settings_and_includes_locked_safeguard() {
        let mut app = App::new();
        app.add_systems(Update, update_readout);
        let setting = app
            .world_mut()
            .spawn((
                DeskSetting,
                Toggle {
                    checked: false,
                    disabled: false,
                    press_consumed: false,
                },
            ))
            .id();
        app.world_mut().spawn((
            DeskSetting,
            Toggle {
                checked: true,
                disabled: true,
                press_consumed: false,
            },
        ));
        let readout = app
            .world_mut()
            .spawn((SafeguardReadout, Text::new("")))
            .id();
        app.update();
        assert_eq!(
            app.world().get::<Text>(readout).unwrap().0,
            "1 / 2 safeguards active"
        );
        app.world_mut().get_mut::<Toggle>(setting).unwrap().checked = true;
        app.update();
        assert_eq!(
            app.world().get::<Text>(readout).unwrap().0,
            "2 / 2 safeguards active"
        );
    }
}
