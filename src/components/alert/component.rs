use crate::components::text::{TextRole, ThemedText};
use crate::components::title::{ThemedTitle, TitleLevel};
use crate::icons::{Icon, IconCommands};
use bevy::prelude::*;
use std::time::Duration;

use crate::primitives::semantic::{SemanticNode, SemanticRole};
use crate::rendering::{Paint, Surface};
use crate::theme::{ThemeColors, ThemeMode, ThemeResource};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlertVariant {
    /// White in light mode, dark in dark mode.
    #[default]
    Default,
    Primary,
    Secondary,
    Success,
    Danger,
    Error,
    Warning,
    Info,
    Light,
    Dark,
    Status,
}

impl AlertVariant {
    const BLUE: Color = Color::srgb(0.05, 0.43, 0.99);
    const TEAL: Color = Color::srgb(0.13, 0.79, 0.59);
    const GREEN: Color = Color::srgb(0.10, 0.70, 0.25);
    const ORANGE: Color = Color::srgb(1.0, 0.49, 0.08);

    /// Resolves `Default` to `Light` or `Dark` for the active theme mode.
    pub fn resolve(self, mode: ThemeMode) -> Self {
        match (self, mode) {
            (Self::Default, ThemeMode::Dark) => Self::Dark,
            (Self::Default, ThemeMode::Light) => Self::Light,
            (other, _) => other,
        }
    }

    pub fn feather_name(&self) -> &'static str {
        match self {
            Self::Default | Self::Primary | Self::Secondary | Self::Light | Self::Dark => "info",
            Self::Success => "check-circle",
            Self::Danger | Self::Error => "x-circle",
            Self::Warning => "alert-triangle",
            Self::Info => "info",
            Self::Status => "wifi",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::Primary => "Primary",
            Self::Secondary => "Secondary",
            Self::Danger => "Danger",
            Self::Light => "Light",
            Self::Dark => "Dark",
            Self::Success => "Success",
            Self::Error => "Error",
            Self::Warning => "Warning",
            Self::Info => "Info",
            Self::Status => "Status",
        }
    }

    pub fn background_color(&self, colors: ThemeColors) -> Color {
        match self {
            Self::Primary => Self::BLUE.with_alpha(0.28),
            Self::Secondary => colors.text_muted.with_alpha(0.28),
            Self::Light | Self::Default => Color::WHITE,
            Self::Dark => Color::srgb_u8(23, 23, 23),
            Self::Success => Self::GREEN.with_alpha(0.28),
            Self::Danger | Self::Error => colors.error.with_alpha(0.26),
            Self::Warning => Self::ORANGE.with_alpha(0.32),
            Self::Info => Self::TEAL.with_alpha(0.28),
            Self::Status => colors.secondary.with_alpha(0.7),
        }
    }

    pub fn accent_color(&self, colors: ThemeColors) -> Color {
        match self {
            Self::Primary => Self::BLUE,
            Self::Secondary => colors.text_muted,
            Self::Light | Self::Default => Color::BLACK,
            Self::Dark => Color::WHITE,
            Self::Success => Self::GREEN,
            Self::Danger | Self::Error => colors.error,
            Self::Warning => Self::ORANGE,
            Self::Info => Self::TEAL,
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

impl Default for Alert {
    fn default() -> Self {
        Self::new(AlertVariant::default(), "")
    }
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

    pub fn primary(message: impl Into<String>) -> Self {
        Self::new(AlertVariant::Primary, message)
    }

    pub fn secondary(message: impl Into<String>) -> Self {
        Self::new(AlertVariant::Secondary, message)
    }

    pub fn danger(message: impl Into<String>) -> Self {
        Self::new(AlertVariant::Danger, message)
    }

    pub fn light(message: impl Into<String>) -> Self {
        Self::new(AlertVariant::Light, message)
    }

    pub fn dark(message: impl Into<String>) -> Self {
        Self::new(AlertVariant::Dark, message)
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

/// Composites a translucent tint over the page so the fill can be fully opaque.
fn flatten(tint: Color, page: Color) -> Color {
    let tint = tint.to_srgba();
    let page = page.to_srgba();
    let a = tint.alpha;
    Color::srgb(
        tint.red * a + page.red * (1.0 - a),
        tint.green * a + page.green * (1.0 - a),
        tint.blue * a + page.blue * (1.0 - a),
    )
}

/// Darkens `accent` until it reaches WCAG AA (4.5:1) against the opaque `backdrop`.
fn readable_accent_text(accent: Color, backdrop: Color) -> Color {
    let accent = accent.to_srgba();
    let mut text = Color::srgb(accent.red, accent.green, accent.blue);
    for step in 1..=20 {
        if crate::theme::meets_contrast(text, backdrop, crate::theme::WCAG_AA_NORMAL_TEXT) {
            break;
        }
        let k = 1.0 - step as f32 * 0.05;
        text = Color::srgb(accent.red * k, accent.green * k, accent.blue * k);
    }
    text
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
        let variant = alert.variant.resolve(theme.current.mode);
        let background = flatten(variant.background_color(colors), colors.background);
        let accent = variant.accent_color(colors);
        let border = match variant {
            AlertVariant::Light => Color::srgb_u8(212, 212, 212),
            AlertVariant::Dark => Color::srgb_u8(38, 38, 38), // shadcn neutral-800
            _ => accent,
        };
        let (title_color, body_color) = match variant {
            AlertVariant::Light => (Color::BLACK, Color::BLACK),
            AlertVariant::Dark => (Color::WHITE, Color::WHITE),
            _ => {
                let text = readable_accent_text(accent, background);
                (text, text)
            }
        };

        root.insert((
            AlertRoot,
            SemanticNode::new(
                if matches!(variant, AlertVariant::Error | AlertVariant::Danger | AlertVariant::Warning) {
                    SemanticRole::Alert
                } else {
                    SemanticRole::Status
                },
            )
            .label(
                alert
                    .title
                    .clone()
                    .unwrap_or_else(|| variant.label().to_string()),
            )
            .description(alert.message.clone()),
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
            .uniform_border(1.0, Paint::solid(border)),
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
                        Icon::feather(variant.feather_name()),
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
                            ThemedTitle::new(TitleLevel::H5).color(title_color),
                            Text::new(title.clone()),
                            TextFont {
                                font_size: FontSize::Px(15.0),
                                ..default()
                            },
                            TextColor(title_color),
                        ));
                    }

                    parent.spawn((
                        AlertMessage,
                        ThemedText::new(TextRole::Body).color(body_color),
                        Text::new(alert.message.clone()),
                        TextFont {
                            font_size: FontSize::Px(14.0),
                            ..default()
                        },
                        TextColor(body_color),
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
                        button.spawn_icon_colored(Icon::feather("x"), 18.0, body_color);
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
