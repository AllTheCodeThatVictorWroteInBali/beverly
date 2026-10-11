//! Slider demo: fine-tune the snack desk's toast order.
//! Run with `-- dark` to start in dark mode.

use beverly::components::slider::{Slider, SliderChanged, SliderStyle, spawn_slider};
use beverly::components::text::{TextRole, ThemedText};
use beverly::components::title::{ThemedTitle, TitleLevel};
use beverly::prelude::*;
use bevy::prelude::*;

#[derive(Component, Clone, Copy, PartialEq)]
enum ToastSetting {
    Browning,
    Slices,
    Deadline,
}

#[derive(Component)]
struct Readout(ToastSetting);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BeverlyPlugin)
        .insert_resource(ThemeResource {
            current: theme_from_cli_args(),
        })
        .add_systems(Startup, setup)
        .add_systems(Update, update_readouts)
        .run();
}

fn setup(mut commands: Commands, theme: Res<ThemeResource>) {
    spawn_themed_page(&mut commands, |root| {
        root.spawn(Node {
            width: Val::Px(480.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            row_gap: Val::Px(24.0),
            ..default()
        })
        .with_children(|content| {
            content.spawn((ThemedTitle::new(TitleLevel::H2), Text::new("Toast Control")));
            content.spawn((
                ThemedText::new(TextRole::Muted),
                Text::new("Breakfast has been escalated to a precision operation."),
            ));
            slider_row(
                content,
                &theme,
                ToastSetting::Browning,
                "Browning level",
                Slider::new(0.0, 100.0).value(42.0),
            );
            slider_row(
                content,
                &theme,
                ToastSetting::Slices,
                "Emergency toast supply",
                Slider::new(1.0, 8.0).value(3.0).step(1.0),
            );
            slider_row(
                content,
                &theme,
                ToastSetting::Deadline,
                "Breakfast deadline",
                Slider::new(0.0, 60.0).value(15.0).step(5.0).disabled(true),
            );
        });
    });
}

fn slider_row(
    parent: &mut ChildSpawnerCommands,
    theme: &ThemeResource,
    setting: ToastSetting,
    label: &str,
    slider: Slider,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|row| {
            row.spawn((ThemedText::new(TextRole::Label), Text::new(label)));
            row.spawn((
                Readout(setting),
                ThemedText::new(TextRole::Muted),
                Text::new(readout(setting, slider.value)),
            ));
            let entity = spawn_slider(
                row,
                slider.label(label),
                SliderStyle {
                    width: 460.0,
                    track_color: theme.current.colors.border,
                    fill_color: theme.current.colors.primary,
                    thumb_color: theme.current.colors.surface_elevated,
                    ..default()
                },
                theme,
            );
            row.commands().entity(entity).insert(setting);
        });
}

fn readout(setting: ToastSetting, value: f32) -> String {
    match setting {
        ToastSetting::Browning => {
            let verdict = if value < 25.0 {
                "Bread with a warm handshake"
            } else if value < 65.0 {
                "Respectably golden"
            } else if value < 90.0 {
                "Crunch with confidence"
            } else {
                "The smoke alarm has opinions"
            };
            format!("{value:.0}% - {verdict}")
        }
        ToastSetting::Slices => format!(
            "{value:.0} slice{} reserved for emergencies",
            if value == 1.0 { "" } else { "s" }
        ),
        ToastSetting::Deadline => format!("{value:.0} minutes - locked by the breakfast committee"),
    }
}

fn update_readouts(
    mut changes: MessageReader<SliderChanged>,
    settings: Query<&ToastSetting>,
    mut readouts: Query<(&Readout, &mut Text)>,
) {
    for change in changes.read() {
        let Ok(setting) = settings.get(change.entity) else {
            continue;
        };
        for (readout_setting, mut text) in &mut readouts {
            if readout_setting.0 == *setting {
                *text = Text::new(readout(*setting, change.value));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slider_changes_update_only_the_matching_readout() {
        let mut app = App::new();
        app.add_message::<SliderChanged>()
            .add_systems(Update, update_readouts);
        let slider = app.world_mut().spawn(ToastSetting::Browning).id();
        let browning = app
            .world_mut()
            .spawn((Readout(ToastSetting::Browning), Text::new("before")))
            .id();
        let slices = app
            .world_mut()
            .spawn((Readout(ToastSetting::Slices), Text::new("unchanged")))
            .id();
        app.world_mut().write_message(SliderChanged {
            entity: slider,
            value: 95.0,
            dragging: true,
        });
        app.update();
        assert_eq!(
            app.world().get::<Text>(browning).unwrap().0,
            "95% - The smoke alarm has opinions"
        );
        assert_eq!(app.world().get::<Text>(slices).unwrap().0, "unchanged");
    }
}
