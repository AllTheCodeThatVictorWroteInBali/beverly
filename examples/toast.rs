//! Toast demo: dispatches from the snack desk.
//! Run with `-- dark` to start in dark mode.

use beverly::components::button::{BeverlyButton, ButtonChild};
use beverly::components::text::{TextRole, ThemedText};
use beverly::components::title::{ThemedTitle, TitleLevel};
use beverly::components::toast::{Toast, ToastKind, ToastPosition};
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
            width: Val::Px(480.0),
            max_width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(18.0),
            ..default()
        })
        .with_children(|content| {
            content.spawn((
                ThemedTitle::new(TitleLevel::H2),
                Text::new("Snack Desk Dispatches"),
            ));
            content.spawn((
                ThemedText::new(TextRole::Muted),
                Text::new("Breaking news from the biscuit tin."),
            ));
            content.spawn(
                BeverlyButton::standard("Secure the biscuits")
                    .children([
                        ButtonChild::icon("check-circle"),
                        ButtonChild::text("Secure the biscuits"),
                    ])
                    .on("click", |commands, _| {
                        show_toast(commands, ToastKind::Success)
                    }),
            );
            content.spawn(
                BeverlyButton::standard("Check the tea bulletin")
                    .children([
                        ButtonChild::icon("info"),
                        ButtonChild::text("Check the tea bulletin"),
                    ])
                    .on("click", |commands, _| show_toast(commands, ToastKind::Info)),
            );
            content.spawn(
                BeverlyButton::standard("Inspect the reserves")
                    .children([
                        ButtonChild::icon("alert-triangle"),
                        ButtonChild::text("Inspect the reserves"),
                    ])
                    .on("click", |commands, _| {
                        show_toast(commands, ToastKind::Warning)
                    }),
            );
            content.spawn(
                BeverlyButton::standard("Investigate the jam")
                    .children([
                        ButtonChild::icon("x-circle"),
                        ButtonChild::text("Investigate the jam"),
                    ])
                    .on("click", |commands, _| {
                        show_toast(commands, ToastKind::Error)
                    }),
            );
            content.spawn(
                BeverlyButton::standard("Consult the kettle")
                    .children([
                        ButtonChild::icon("coffee"),
                        ButtonChild::text("Consult the kettle"),
                    ])
                    .on("click", |commands, _| {
                        show_toast(commands, ToastKind::Loading)
                    }),
            );
        });
    });
}

fn notification(kind: ToastKind) -> Toast {
    let (title, message) = match kind {
        ToastKind::Success => (
            "Biscuits secured",
            "The emergency tin is fully stocked. Quality assurance ate only one.",
        ),
        ToastKind::Info => (
            "Tea bulletin",
            "Second breakfast has been approved by an overwhelming majority.",
        ),
        ToastKind::Warning => (
            "Reserve levels critical",
            "Two biscuits remain. One is already under negotiation.",
        ),
        ToastKind::Error => (
            "Jam incident",
            "The lid is stuck. Spread Operations has declared a sticky situation.",
        ),
        ToastKind::Loading => (
            "Kettle deliberating",
            "Your tea request is receiving a very thorough review.",
        ),
    };
    Toast::new(kind, message)
        .title(title)
        .position(match kind {
            ToastKind::Success => ToastPosition::TopLeft,
            ToastKind::Info => ToastPosition::TopCenter,
            ToastKind::Warning => ToastPosition::BottomLeft,
            ToastKind::Error => ToastPosition::BottomRight,
            ToastKind::Loading => ToastPosition::BottomCenter,
        })
        .duration(if kind == ToastKind::Loading { 0 } else { 5000 })
        .close_label("Dismiss snack notification")
}

fn show_toast(commands: &mut Commands, kind: ToastKind) {
    commands.queue(move |world: &mut World| {
        world.spawn(notification(kind));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notification_kinds_have_distinct_positions() {
        let kinds = [
            ToastKind::Success,
            ToastKind::Info,
            ToastKind::Warning,
            ToastKind::Error,
            ToastKind::Loading,
        ];
        let positions: Vec<_> = kinds
            .into_iter()
            .map(|kind| notification(kind).position)
            .collect();
        for (index, position) in positions.iter().enumerate() {
            assert!(!positions[..index].contains(position));
        }
    }

    #[test]
    fn notifications_coexist_and_loading_waits_for_dismissal() {
        let mut world = World::new();
        show_toast(&mut world.commands(), ToastKind::Success);
        world.flush();
        let mut toasts = world.query::<&Toast>();
        assert_eq!(toasts.single(&world).unwrap().kind, ToastKind::Success);
        show_toast(&mut world.commands(), ToastKind::Loading);
        world.flush();
        assert_eq!(toasts.iter(&world).count(), 2);
        let toast = toasts
            .iter(&world)
            .find(|toast| toast.kind == ToastKind::Loading)
            .unwrap();
        assert_eq!(toast.kind, ToastKind::Loading);
        assert!(toast.duration.is_none());
        assert!(toast.timer.is_none());
        assert!(toast.dismissible);
        assert_eq!(toast.position, ToastPosition::BottomCenter);
    }
}
