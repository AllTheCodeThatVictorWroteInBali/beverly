use crate::components::text::{TextRole, ThemedText};
use crate::components::title::{ThemedTitle, TitleLevel};
use crate::icons::{Icon, IconCommands};
use crate::primitives::semantic::{SemanticNode, SemanticRole};
use bevy::prelude::*;
use std::time::Duration;

use crate::primitives::semantic::AnnouncementPriority;
use crate::rendering::{Paint, Surface};
use crate::theme::{ThemeColors, ThemeResource};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastKind {
    Success,
    Info,
    Warning,
    Error,
    Loading,
}

impl ToastKind {
    pub fn feather_name(&self) -> &'static str {
        match self {
            Self::Success => "check-circle",
            Self::Info => "info",
            Self::Warning => "alert-triangle",
            Self::Error => "x-circle",
            Self::Loading => "loader",
        }
    }

    pub fn accent_color(&self, colors: ThemeColors) -> Color {
        match self {
            Self::Success => colors.success,
            Self::Info => colors.info,
            Self::Warning => colors.warning,
            Self::Error => colors.error,
            Self::Loading => colors.primary,
        }
    }
}

/// Where a Toast anchors itself within the viewport.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastPosition {
    TopLeft,
    TopCenter,
    TopRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

impl ToastPosition {
    fn is_top(&self) -> bool {
        matches!(self, Self::TopLeft | Self::TopCenter | Self::TopRight)
    }

    fn justify_content(&self) -> JustifyContent {
        match self {
            Self::TopLeft | Self::BottomLeft => JustifyContent::FlexStart,
            Self::TopCenter | Self::BottomCenter => JustifyContent::Center,
            Self::TopRight | Self::BottomRight => JustifyContent::FlexEnd,
        }
    }
}

/// Default time a Toast remains visible before it starts to close.
pub const DEFAULT_TOAST_DURATION: Duration = Duration::from_millis(5000);

/// Reusable Toast component: a transient notification with its own
/// presentation lifecycle (position, duration, dismissal).
///
/// The component itself contains the state/configuration. `ToastPlugin`
/// handles rendering, the visibility timer, and dismissal.
#[derive(Component, Debug, Clone)]
pub struct Toast {
    pub kind: ToastKind,

    pub title: Option<String>,
    pub message: String,

    pub position: ToastPosition,

    /// Whether the user can dismiss the Toast with the close control.
    pub dismissible: bool,
    /// Accessible name for the close control.
    pub close_label: Option<String>,

    /// Automatically remove after this duration. `None` means the Toast
    /// stays until dismissed.
    pub duration: Option<Duration>,

    /// Internal lifetime timer.
    #[allow(dead_code)]
    pub timer: Option<Timer>,
}

/// Fluent section content attached to a Toast.
#[derive(Component, Clone, Default)]
pub struct ToastSections {
    pub header: Vec<crate::primitives::composition::UiElement>,
    pub body: Vec<crate::primitives::composition::UiElement>,
    pub footer: Vec<crate::primitives::composition::UiElement>,
}

impl Toast {
    pub fn new(kind: ToastKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            title: None,
            message: message.into(),
            position: ToastPosition::BottomRight,
            dismissible: true,
            close_label: None,
            duration: Some(DEFAULT_TOAST_DURATION),
            timer: Some(Timer::new(DEFAULT_TOAST_DURATION, TimerMode::Once)),
        }
    }

    pub fn success(message: impl Into<String>) -> Self {
        Self::new(ToastKind::Success, message)
    }

    pub fn info(message: impl Into<String>) -> Self {
        Self::new(ToastKind::Info, message)
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(ToastKind::Warning, message)
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::new(ToastKind::Error, message)
    }

    pub fn loading(message: impl Into<String>) -> Self {
        let mut toast = Self::new(ToastKind::Loading, message);
        toast.duration = None;
        toast.timer = None;
        toast
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn position(mut self, position: ToastPosition) -> Self {
        self.position = position;
        self
    }

    pub fn dismissible(mut self, value: bool) -> Self {
        self.dismissible = value;
        self
    }

    pub fn close_label(mut self, label: impl Into<String>) -> Self {
        self.close_label = Some(label.into());
        self
    }

    /// Sets how long the Toast remains visible, in milliseconds. Pass `0` to
    /// keep the Toast visible until it is dismissed.
    pub fn duration(mut self, millis: u64) -> Self {
        if millis == 0 {
            self.duration = None;
            self.timer = None;
        } else {
            let duration = Duration::from_millis(millis);
            self.duration = Some(duration);
            self.timer = Some(Timer::new(duration, TimerMode::Once));
        }
        self
    }
}

pub fn basic_toasts() -> Vec<Toast> {
    vec![
        Toast::success("Your changes were saved successfully.").title("Saved"),
        Toast::info("A new version is ready to review.").title("New update"),
        Toast::warning("You have edits that have not been published yet.").title("Unsaved changes"),
        Toast::error("The last action could not be completed.").title("Action failed"),
        Toast::loading("We are updating your data in the background.").title("Syncing"),
    ]
}

pub fn error_toasts() -> Vec<Toast> {
    vec![
        Toast::error("We could not upload the file. Check your connection and try again.")
            .title("Upload failed"),
        Toast::error("Your changes were not saved. Retry after the connection stabilizes.")
            .title("Save failed"),
        Toast::error("Please fill in all required fields before continuing.")
            .title("Validation error"),
        Toast::error("We could not reach the server. Please try again in a moment.")
            .title("Server unavailable"),
    ]
}

/// Marker for the positioning lane a Toast anchors itself to within the
/// viewport (top/bottom edge, left/center/right aligned).
#[derive(Component)]
struct ToastLane;

/// Marker for the visible Toast card, nested inside the positioning lane.
#[derive(Component)]
struct ToastCard;

/// Marker for the dismiss button.
#[derive(Component)]
struct ToastDismissButton;

pub struct ToastPlugin;

impl Plugin for ToastPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (spawn_toast_ui, update_toast_timers, dismiss_toasts),
        );
    }
}

/// Creates the visual representation of newly-added Toasts.
///
/// The Toast entity itself becomes the positioning lane (an absolutely
/// positioned, full-width strip anchored to the top or bottom edge of the
/// viewport); the visible card is a single child within it so empty space in
/// the lane does not block interaction with whatever is underneath.
fn spawn_toast_ui(
    mut commands: Commands,
    toasts: Query<(Entity, &Toast, Option<&ToastSections>), Added<Toast>>,
    theme: Res<ThemeResource>,
) {
    let colors = theme.current.colors;

    for (entity, toast, sections) in &toasts {
        let accent = toast.kind.accent_color(colors);

        let mut root = commands.entity(entity);
        let mut semantic = SemanticNode::new(SemanticRole::Status)
            .label(
                toast
                    .title
                    .clone()
                    .unwrap_or_else(|| "Notification".to_string()),
            )
            .description(toast.message.clone());
        semantic.live = Some(AnnouncementPriority::Polite);

        root.insert((
            ToastLane,
            semantic,
            Node {
                display: Display::Flex,
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: if toast.position.is_top() {
                    Val::Px(16.0)
                } else {
                    Val::Auto
                },
                bottom: if toast.position.is_top() {
                    Val::Auto
                } else {
                    Val::Px(16.0)
                },
                justify_content: toast.position.justify_content(),
                padding: UiRect::horizontal(Val::Px(16.0)),
                ..default()
            },
            Pickable::IGNORE,
            BackgroundColor(Color::NONE),
            ZIndex(1000),
        ));

        root.with_children(|lane| {
            lane.spawn((
                ToastCard,
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    max_width: Val::Px(360.0),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(12.0)),
                    padding: UiRect::all(Val::Px(14.0)),
                    column_gap: Val::Px(12.0),
                    ..default()
                },
                BackgroundColor(Color::NONE),
                BorderColor::all(Color::NONE),
                Surface::rounded_rect_fill(12.0, Paint::solid(colors.surface_elevated))
                    .uniform_border(1.0, Paint::solid(accent.with_alpha(0.35))),
            ))
            .with_children(|card| {
                // Icon
                card.spawn((Node {
                    width: Val::Px(20.0),
                    height: Val::Px(20.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },))
                    .with_children(|icon| {
                        icon.spawn_icon_colored(
                            Icon::feather(toast.kind.feather_name()),
                            20.0,
                            accent,
                        );
                    });

                // Content
                card.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    ..default()
                })
                .with_children(|content| {
                    if let Some(sections) = sections {
                        spawn_toast_section(content, sections.header.clone(), "toast-header");
                    }

                    if let Some(title) = &toast.title {
                        content.spawn((
                            ThemedTitle::new(TitleLevel::H5),
                            Text::new(title.clone()),
                            TextFont {
                                font_size: FontSize::Px(15.0),
                                ..default()
                            },
                            TextColor(colors.text),
                        ));
                    }

                    content.spawn((
                        ThemedText::new(TextRole::Body),
                        Text::new(toast.message.clone()),
                        TextFont {
                            font_size: FontSize::Px(14.0),
                            ..default()
                        },
                        TextColor(colors.text_muted),
                    ));

                    if let Some(sections) = sections {
                        spawn_toast_section(content, sections.body.clone(), "toast-body");
                    }
                });

                if let Some(sections) = sections {
                    spawn_toast_section(card, sections.footer.clone(), "toast-footer");
                }

                // Dismiss button
                if toast.dismissible {
                    let close_label = toast
                        .close_label
                        .clone()
                        .unwrap_or_else(|| "Dismiss notification".to_string());

                    card.spawn((
                        ToastDismissButton,
                        Button,
                        SemanticNode::new(SemanticRole::Button).label(close_label),
                        Node {
                            width: Val::Px(28.0),
                            height: Val::Px(28.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                    ))
                    .with_children(|button| {
                        button.spawn_icon_colored(Icon::feather("x"), 18.0, colors.text_muted);
                    });
                }
            });
        });
    }
}

fn spawn_toast_section(
    parent: &mut ChildSpawnerCommands,
    children: Vec<crate::primitives::composition::UiElement>,
    name: &'static str,
) {
    if children.is_empty() {
        return;
    }

    parent
        .spawn((
            Name::new(name),
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(6),
                ..default()
            },
        ))
        .with_children(|section| {
            for child in children {
                child.spawn(section);
            }
        });
}

fn update_toast_timers(
    time: Res<Time>,
    mut commands: Commands,
    mut toasts: Query<(Entity, &mut Toast)>,
) {
    for (entity, mut toast) in &mut toasts {
        if let Some(timer) = &mut toast.timer {
            timer.tick(time.delta());

            if timer.is_finished() {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn dismiss_toasts(
    mut commands: Commands,
    interactions: Query<(&Interaction, &ChildOf), (Changed<Interaction>, With<ToastDismissButton>)>,
    cards: Query<&ChildOf, With<ToastCard>>,
) {
    for (interaction, card_parent) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        // interaction is on the dismiss button, whose parent is the card,
        // whose parent is the Toast/lane entity itself.
        let Ok(lane_parent) = cards.get(card_parent.0) else {
            continue;
        };

        commands.entity(lane_parent.0).despawn();
    }
}
