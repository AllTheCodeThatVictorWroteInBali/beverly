use crate::icons::{Icon, IconCommands};
use crate::components::text::{TextRole, ThemedText};
use crate::components::title::{ThemedTitle, TitleLevel};
use bevy::prelude::*;
use std::time::Duration;

use crate::rendering::{Paint, Surface};
use crate::theme::{ThemeColors, ThemeResource};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertVariant {
    Success,
    Error,
    Warning,
    Info,
    Status,
}

impl AlertVariant {
    pub fn feather_name(&self) -> &'static str {
        match self {
            Self::Success => "check-circle",
            Self::Error => "x-circle",
            Self::Warning => "alert-triangle",
            Self::Info => "info",
            Self::Status => "wifi",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Success => "Success",
            Self::Error => "Error",
            Self::Warning => "Warning",
            Self::Info => "Info",
            Self::Status => "Status",
        }
    }

    pub fn background_color(&self, colors: ThemeColors) -> Color {
        match self {
            Self::Success => colors.success.with_alpha(0.11),
            Self::Error => colors.error.with_alpha(0.10),
            Self::Warning => colors.warning.with_alpha(0.14),
            Self::Info => colors.info.with_alpha(0.11),
            Self::Status => colors.secondary.with_alpha(0.7),
        }
    }

    pub fn accent_color(&self, colors: ThemeColors) -> Color {
        match self {
            Self::Success => colors.success,
            Self::Error => colors.error,
            Self::Warning => colors.warning,
            Self::Info => colors.info,
            Self::Status => colors.primary,
        }
    }
}

/// Reusable alert component.
///
/// The component itself contains the state/configuration.
/// `AlertPlugin` handles rendering, animation and dismissal.
#[derive(Component, Debug, Clone)]
pub struct Alert {
    pub variant: AlertVariant,

    pub title: Option<String>,
    pub message: String,

    pub dismissible: bool,

    /// Automatically remove after this duration.
    pub duration: Option<Duration>,

    /// Internal lifetime timer.
    #[allow(dead_code)]
    pub timer: Option<Timer>,
}

impl Alert {
    pub fn new(variant: AlertVariant, message: impl Into<String>) -> Self {
        Self {
            variant,
            title: None,
            message: message.into(),
            dismissible: true,
            duration: None,
            timer: None,
        }
    }

    pub fn success(message: impl Into<String>) -> Self {
        Self::new(AlertVariant::Success, message)
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::new(AlertVariant::Error, message)
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(AlertVariant::Warning, message)
    }

    pub fn info(message: impl Into<String>) -> Self {
        Self::new(AlertVariant::Info, message)
    }

    pub fn status(message: impl Into<String>) -> Self {
        Self::new(AlertVariant::Status, message)
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn dismissible(mut self, value: bool) -> Self {
        self.dismissible = value;
        self
    }

    pub fn duration(mut self, duration: Duration) -> Self {
        self.timer = Some(Timer::new(duration, TimerMode::Once));
        self.duration = Some(duration);
        self
    }
}

/// Marker for the root UI node of an alert.
#[derive(Component)]
struct AlertRoot;

/// Marker for the alert message text.
#[derive(Component)]
struct AlertMessage;

/// Marker for the alert title.
#[derive(Component)]
struct AlertTitle;

/// Marker for the alert icon.
#[derive(Component)]
struct AlertIcon;

/// Marker for the dismiss button.
#[derive(Component)]
struct AlertDismissButton;

pub struct AlertPlugin;

impl Plugin for AlertPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (spawn_alert_ui, update_alert_timers, dismiss_alerts),
        );
    }
}

/// Creates the visual representation of newly-added alerts.
fn spawn_alert_ui(
    mut commands: Commands,
    alerts: Query<(Entity, &Alert), Added<Alert>>,
    theme: Res<ThemeResource>,
) {
    let colors = theme.current.colors;

    for (entity, alert) in &alerts {
        let mut root = commands.entity(entity);
        let background = alert.variant.background_color(colors);
        let accent = alert.variant.accent_color(colors);

        root.insert((
            AlertRoot,
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(12.0)),
                padding: UiRect::all(Val::Px(14.0)),
                column_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(Color::NONE),
            BorderColor::all(Color::NONE),
            Surface::rounded_rect_fill(12.0, Paint::solid(background))
                .uniform_border(1.0, Paint::solid(accent.with_alpha(0.35))),
        ));

        // Icon
        root.with_children(|parent| {
            parent
                .spawn((
                    AlertIcon,
                    Node {
                        width: Val::Px(20.0),
                        height: Val::Px(20.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                ))
                .with_children(|icon| {
                    icon.spawn_icon_colored(
                        Icon::feather(alert.variant.feather_name()),
                        20.0,
                        accent,
                    );
                });

            // Content
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    ..default()
                })
                .with_children(|parent| {
                    if let Some(title) = &alert.title {
                        parent.spawn((
                            AlertTitle,
                            ThemedTitle::new(TitleLevel::H5),
                            Text::new(title.clone()),
                            TextFont {
                                font_size: FontSize::Px(15.0),
                                ..default()
                            },
                            TextColor(colors.text),
                        ));
                    }

                    parent.spawn((
                        AlertMessage,
                        ThemedText::new(TextRole::Body),
                        Text::new(alert.message.clone()),
                        TextFont {
                            font_size: FontSize::Px(14.0),
                            ..default()
                        },
                        TextColor(colors.text_muted),
                    ));
                });

            // Dismiss button
            if alert.dismissible {
                parent
                    .spawn((
                        AlertDismissButton,
                        Button,
                        Node {
                            width: Val::Px(28.0),
                            height: Val::Px(28.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                    ))
                    .with_children(|button| {
                        button.spawn_icon_colored(
                            Icon::feather("x"),
                            18.0,
                            colors.text_muted,
                        );
                    });
            }
        });
    }
}

fn update_alert_timers(
    time: Res<Time>,
    mut commands: Commands,
    mut alerts: Query<(Entity, &mut Alert)>,
) {
    for (entity, mut alert) in &mut alerts {
        if let Some(timer) = &mut alert.timer {
            timer.tick(time.delta());

            if timer.is_finished() {
                commands
                    .entity(entity)
                    .despawn_related::<Children>()
                    .despawn();
            }
        }
    }
}

fn dismiss_alerts(
    mut commands: Commands,
    interactions: Query<(&Interaction, &ChildOf), (Changed<Interaction>, With<AlertDismissButton>)>,
) {
    for (interaction, parent) in &interactions {
        if *interaction == Interaction::Pressed {
            commands
                .entity(parent.0)
                .despawn_related::<Children>()
                .despawn();
        }
    }
}
