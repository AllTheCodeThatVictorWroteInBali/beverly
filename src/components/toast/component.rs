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

#[derive(Component, Default)]
struct ToastAnimation {
    exit: Option<f32>,
}

#[derive(Component, Default)]
struct ToastStackMotion {
    current: Option<f32>,
    from: f32,
    target: f32,
    elapsed: f32,
}

const TOAST_SLIDE_SECONDS: f32 = 0.28;
const TOAST_FADE_SECONDS: f32 = 0.24;

#[derive(Component)]
struct ToastStackOrder(u64);

const TOAST_STACK_GAP: f32 = 12.0;

pub struct ToastPlugin;

impl Plugin for ToastPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (spawn_toast_ui, update_toast_timers, dismiss_toasts).chain(),
        )
        .add_systems(
            PostUpdate,
            (stack_toasts, animate_toasts)
                .chain()
                .before(bevy::ui::UiSystems::Prepare),
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
    mut next_order: Local<u64>,
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
            ToastStackOrder(*next_order),
            ToastStackMotion::default(),
            Visibility::Hidden,
            ToastAnimation::default(),
            UiTransform::default(),
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
        *next_order += 1;

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

fn stack_toasts(
    time: Res<Time>,
    policy: Res<crate::theme::AccessibilityVisualPolicyResource>,
    mut lanes: Query<
        (
            Entity,
            &Toast,
            &ToastStackOrder,
            Option<&ChildOf>,
            &ComputedNode,
            &mut Node,
            &mut Visibility,
            &mut ToastStackMotion,
        ),
        With<ToastLane>,
    >,
) {
    let mut ordered: Vec<_> = lanes
        .iter()
        .map(|(entity, toast, order, parent, computed, _, _, _)| {
            (
                entity,
                toast.position,
                order.0,
                parent.map(ChildOf::parent),
                computed.size().y * computed.inverse_scale_factor(),
            )
        })
        .collect();
    ordered.sort_by_key(|entry| std::cmp::Reverse(entry.2));
    let mut offsets: Vec<(ToastPosition, Option<Entity>, f32)> = Vec::new();
    for (entity, position, _, parent, height) in ordered {
        let group = if let Some(index) = offsets
            .iter()
            .position(|entry| entry.0 == position && entry.1 == parent)
        {
            index
        } else {
            offsets.push((position, parent, 16.0));
            offsets.len() - 1
        };
        let Ok((_, _, _, _, _, mut node, mut visibility, mut motion)) = lanes.get_mut(entity)
        else {
            continue;
        };
        *visibility = if height > 0.0 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if height <= 0.0 {
            continue;
        }
        let target = offsets[group].2;
        if motion.current.is_none() {
            motion.current = Some(-height - TOAST_STACK_GAP);
            motion.target = f32::NAN;
        }
        if motion.target != target {
            motion.from = motion.current.unwrap();
            motion.target = target;
            motion.elapsed = 0.0;
        }
        motion.elapsed = (motion.elapsed + time.delta_secs()).min(TOAST_SLIDE_SECONDS);
        let progress = if policy.current.reduced_motion {
            1.0
        } else {
            motion.elapsed / TOAST_SLIDE_SECONDS
        };
        let offset = motion.from + (target - motion.from) * (1.0 - (1.0 - progress).powi(3));
        motion.current = Some(offset);
        let top = if position.is_top() {
            Val::Px(offset)
        } else {
            Val::Auto
        };
        let bottom = if position.is_top() {
            Val::Auto
        } else {
            Val::Px(offset)
        };
        if node.top != top {
            node.top = top;
        }
        if node.bottom != bottom {
            node.bottom = bottom;
        }
        offsets[group].2 += height + TOAST_STACK_GAP;
    }
}

fn update_toast_timers(time: Res<Time>, mut toasts: Query<(&mut Toast, &mut ToastAnimation)>) {
    for (mut toast, mut animation) in &mut toasts {
        if let Some(timer) = &mut toast.timer {
            timer.tick(time.delta());

            if timer.is_finished() {
                animation.exit.get_or_insert(0.0);
            }
        }
    }
}

fn dismiss_toasts(
    mut animations: Query<&mut ToastAnimation>,
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

        if let Ok(mut animation) = animations.get_mut(lane_parent.0) {
            animation.exit.get_or_insert(0.0);
        }
    }
}

#[cfg(test)]
mod animation_tests {
    use super::*;

    #[test]
    fn new_toast_enters_from_outside_and_pushes_existing_toast_inward() {
        for position in [ToastPosition::TopLeft, ToastPosition::BottomRight] {
            let mut app = App::new();
            app.init_resource::<Time>()
                .init_resource::<crate::theme::AccessibilityVisualPolicyResource>()
                .add_systems(Update, stack_toasts);
            let spawn = |world: &mut World, order, height| {
                world
                    .spawn((
                        Toast::info("Tea").position(position),
                        ToastLane,
                        ToastStackOrder(order),
                        ToastStackMotion::default(),
                        ComputedNode {
                            size: Vec2::new(360.0, height),
                            ..default()
                        },
                        Node::default(),
                        Visibility::Hidden,
                    ))
                    .id()
            };
            let old = spawn(app.world_mut(), 0, 80.0);
            app.update();
            assert_eq!(
                app.world().get::<ToastStackMotion>(old).unwrap().current,
                Some(-92.0)
            );
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(Duration::from_secs_f32(TOAST_SLIDE_SECONDS));
            app.update();
            assert_eq!(
                app.world().get::<ToastStackMotion>(old).unwrap().current,
                Some(16.0)
            );
            let new = spawn(app.world_mut(), 1, 120.0);
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(Duration::ZERO);
            app.update();
            assert_eq!(
                app.world().get::<ToastStackMotion>(new).unwrap().current,
                Some(-132.0)
            );
            assert_eq!(
                app.world().get::<ToastStackMotion>(old).unwrap().current,
                Some(16.0)
            );
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(Duration::from_millis(140));
            app.update();
            let offset = app
                .world()
                .get::<ToastStackMotion>(old)
                .unwrap()
                .current
                .unwrap();
            assert!(offset > 16.0 && offset < 148.0);
            app.update();
            assert_eq!(
                app.world().get::<ToastStackMotion>(new).unwrap().current,
                Some(16.0)
            );
            assert_eq!(
                app.world().get::<ToastStackMotion>(old).unwrap().current,
                Some(148.0)
            );
        }
    }

    #[test]
    fn same_position_toasts_stack_by_height_and_close_gaps_after_removal() {
        for position in [ToastPosition::TopLeft, ToastPosition::BottomRight] {
            let mut app = App::new();
            app.init_resource::<Time>()
                .insert_resource(crate::theme::AccessibilityVisualPolicyResource {
                    current: crate::theme::AccessibilityVisualPolicy {
                        reduced_motion: true,
                        ..default()
                    },
                })
                .add_systems(Update, stack_toasts);
            let mut entities = Vec::new();
            for (order, height) in [(0, 80.0), (1, 120.0), (2, 60.0)] {
                entities.push(
                    app.world_mut()
                        .spawn((
                            Toast::info("Tea").position(position),
                            ToastLane,
                            ToastStackOrder(order),
                            ToastStackMotion::default(),
                            ComputedNode {
                                size: Vec2::new(360.0, height),
                                ..default()
                            },
                            Node::default(),
                            Visibility::Hidden,
                        ))
                        .id(),
                );
            }
            let other = app
                .world_mut()
                .spawn((
                    Toast::info("Other corner").position(ToastPosition::TopRight),
                    ToastLane,
                    ToastStackOrder(3),
                    ToastStackMotion::default(),
                    ComputedNode {
                        size: Vec2::new(360.0, 80.0),
                        ..default()
                    },
                    Node::default(),
                    Visibility::Hidden,
                ))
                .id();
            app.update();
            for (entity, offset) in entities.iter().zip([220.0, 88.0, 16.0]) {
                let node = app.world().get::<Node>(*entity).unwrap();
                assert_eq!(
                    if position.is_top() {
                        node.top
                    } else {
                        node.bottom
                    },
                    Val::Px(offset)
                );
                assert_eq!(
                    *app.world().get::<Visibility>(*entity).unwrap(),
                    Visibility::Inherited
                );
            }
            assert_eq!(app.world().get::<Node>(other).unwrap().top, Val::Px(16.0));
            app.world_mut().despawn(entities[2]);
            app.update();
            let node = app.world().get::<Node>(entities[1]).unwrap();
            assert_eq!(
                if position.is_top() {
                    node.top
                } else {
                    node.bottom
                },
                Val::Px(16.0)
            );
            let node = app.world().get::<Node>(entities[0]).unwrap();
            assert_eq!(
                if position.is_top() {
                    node.top
                } else {
                    node.bottom
                },
                Val::Px(148.0)
            );
        }
    }

    #[test]
    fn timeout_starts_fade_before_despawning() {
        let mut app = App::new();
        app.init_resource::<Time>()
            .init_resource::<crate::theme::AccessibilityVisualPolicyResource>()
            .add_systems(Update, (update_toast_timers, animate_toasts).chain());
        let toast = app
            .world_mut()
            .spawn((
                Toast::info("Tea ready")
                    .duration(10)
                    .position(ToastPosition::TopLeft),
                ToastAnimation::default(),
                UiTransform::default(),
            ))
            .id();
        let label = app
            .world_mut()
            .spawn((TextColor(Color::WHITE), ChildOf(toast)))
            .id();
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_millis(20));
        app.update();
        assert!(
            app.world()
                .get::<ToastAnimation>(toast)
                .unwrap()
                .exit
                .is_some()
        );
        assert!(app.world().get::<TextColor>(label).unwrap().0.alpha() < 1.0);
        assert!(app.world().get::<TextColor>(label).unwrap().0.alpha() > 0.0);
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_millis(250));
        app.update();
        assert!(app.world().get_entity(toast).is_err());
    }
}

fn animate_toasts(
    time: Res<Time>,
    mut commands: Commands,
    mut toasts: Query<(Entity, &mut ToastAnimation), With<Toast>>,
    parents: Query<&ChildOf>,
    mut visuals: Query<(
        Entity,
        Option<&mut Surface>,
        Option<&mut TextColor>,
        Option<&mut crate::icons::IconNode>,
    )>,
) {
    for (entity, mut animation) in &mut toasts {
        let Some(elapsed) = animation.exit.as_mut() else {
            continue;
        };
        *elapsed += time.delta_secs();
        let alpha = (1.0 - *elapsed / TOAST_FADE_SECONDS).clamp(0.0, 1.0);
        for (visual, surface, text, icon) in &mut visuals {
            let mut ancestor = visual;
            while ancestor != entity {
                let Ok(parent) = parents.get(ancestor) else {
                    break;
                };
                ancestor = parent.parent();
            }
            if ancestor != entity {
                continue;
            }
            if let Some(mut surface) = surface {
                surface.mask =
                    Some(crate::rendering::Mask::new(surface.shape.clone()).with_opacity(alpha));
            }
            if let Some(mut text) = text {
                text.0 = text.0.with_alpha(alpha);
            }
            if let Some(mut icon) = icon {
                icon.color = icon.color.with_alpha(alpha);
            }
        }
        if alpha <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
