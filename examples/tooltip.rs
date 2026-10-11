//! Tooltip demo: small print from the snack desk.
//! Run with `-- dark` to start in dark mode.

use beverly::components::button::{BeverlyButton, ButtonChild};
use beverly::components::text::{TextRole, ThemedText};
use beverly::components::title::{ThemedTitle, TitleLevel};
use beverly::components::tooltip::{Tooltip, TooltipPlacement};
use beverly::prelude::*;
use bevy::prelude::*;

#[derive(Component)]
struct DispatchReadout;

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
            width: Val::Px(540.0),
            max_width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(22.0),
            ..default()
        })
        .with_children(|content| {
            content.spawn((
                ThemedTitle::new(TitleLevel::H2),
                Text::new("Snack Desk Small Print"),
            ));
            content.spawn((
                ThemedText::new(TextRole::Muted),
                Text::new("Every tiny task has a tiny footnote."),
            ));
            content
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(56.0),
                    margin: UiRect::vertical(Val::Px(32.0)),
                    ..default()
                })
                .with_children(build_actions);
            content.spawn((
                DispatchReadout,
                ThemedText::new(TextRole::Body),
                Text::new("The desk is awaiting its next tiny mission."),
            ));
        });
    });
}

fn build_actions(parent: &mut ChildSpawnerCommands) {
    for actions in [
        [
            (
                "archive",
                "Reserve biscuits",
                "One tin. No sharing clause.",
                TooltipPlacement::Top,
            ),
            (
                "coffee",
                "Brew tea",
                "Steeped in responsibility.",
                TooltipPlacement::Bottom,
            ),
        ],
        [
            (
                "search",
                "Inspect crumbs",
                "Evidence, not a snack.",
                TooltipPlacement::Left,
            ),
            (
                "sun",
                "Approve toast",
                "Golden is the official standard.",
                TooltipPlacement::Right,
            ),
        ],
    ] {
        parent
            .spawn(Node {
                column_gap: Val::Px(24.0),
                flex_wrap: FlexWrap::Wrap,
                row_gap: Val::Px(24.0),
                justify_content: JustifyContent::Center,
                ..default()
            })
            .with_children(|row| {
                for (icon, label, note, placement) in actions {
                    row.spawn((
                        BeverlyButton::standard(label)
                            .children([ButtonChild::icon(icon), ButtonChild::text(label)])
                            .on("click", log_dispatch),
                        Tooltip::new(note).placement(placement).delay(0.35),
                    ));
                }
            });
    }
}

fn log_dispatch(commands: &mut Commands, _button: Entity) {
    commands.queue(|world: &mut World| {
        let mut readouts = world.query_filtered::<&mut Text, With<DispatchReadout>>();
        for mut text in readouts.iter_mut(world) {
            *text = Text::new("Mission logged. The biscuit committee has been notified.");
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actions_have_four_tooltip_placements_and_a_hover_delay() {
        let mut world = World::new();
        world
            .commands()
            .spawn(Node::default())
            .with_children(build_actions);
        world.flush();
        let mut query = world.query::<(&Tooltip, &BeverlyButton)>();
        let mut placements = [false; 4];
        for (tooltip, button) in query.iter(&world) {
            let index = match tooltip.placement {
                TooltipPlacement::Top => 0,
                TooltipPlacement::Bottom => 1,
                TooltipPlacement::Left => 2,
                TooltipPlacement::Right => 3,
            };
            placements[index] = true;
            assert_eq!(tooltip.delay, 0.35);
            assert!(!tooltip.visible);
            assert!(!button.label.is_empty());
        }
        assert_eq!(placements, [true; 4]);
    }
}
