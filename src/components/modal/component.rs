use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use std::collections::HashMap;

use crate::primitives::a11y;
use crate::animation::blur::component::{BackdropBlur, spawn_backdrop_blur};
use crate::primitives::focus::{FocusOrigin, FocusRequest, FocusScope, FocusSystems};
use crate::icons::{Icon, IconNode};
use crate::primitives::interaction::{InteractionAction, InteractionActionEvent};
use crate::components::text::{TextRole, ThemedText};
use crate::theme::{AccessibilityVisualPolicyResource, ThemeResource};
use crate::components::title::{ThemedTitle, TitleLevel};
use crate::rendering::prelude::*;

// ─────────────────────────────────────────────────────────────────────────────
// Modal
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Component, Debug, Clone)]
pub struct Modal {
    pub open: bool,
    pub close_on_backdrop: bool,
    pub close_on_escape: bool,
}

impl Default for Modal {
    fn default() -> Self {
        Self {
            open: false,
            close_on_backdrop: true,
            close_on_escape: true,
        }
    }
}

impl Modal {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(mut self) -> Self {
        self.open = true;
        self
    }

    pub fn closed(mut self) -> Self {
        self.open = false;
        self
    }

    pub fn close_on_backdrop(mut self, enabled: bool) -> Self {
        self.close_on_backdrop = enabled;
        self
    }

    pub fn close_on_escape(mut self, enabled: bool) -> Self {
        self.close_on_escape = enabled;
        self
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Modal Parts
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct ModalOverlay;

#[derive(Component)]
pub struct ModalSurface;

#[derive(Component)]
pub struct ModalHeader;

#[derive(Component)]
pub struct ModalBody;

#[derive(Component)]
pub struct ModalFooter;

#[derive(Component)]
pub struct ModalCloseButton;

#[derive(Component)]
struct ModalActionButton {
    owner: Entity,
}

#[derive(Component)]
struct ModalBaseColor(Color);

/// Remembers which entity had keyboard focus before this modal opened, so it
/// can be restored when the modal closes.
#[derive(Component, Default)]
struct ModalReturnFocus {
    previous: Option<Entity>,
    was_open: bool,
}

// ─────────────────────────────────────────────────────────────────────────────
// Animation
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct ModalAnimation {
    pub opacity: f32,
    pub scale: f32,
}

impl Default for ModalAnimation {
    fn default() -> Self {
        Self {
            opacity: 0.0,
            scale: 0.92,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Events
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Message, Debug, Clone, Copy)]
pub struct ModalOpened {
    pub entity: Entity,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct ModalClosed {
    pub entity: Entity,
}

// ─────────────────────────────────────────────────────────────────────────────
// Style
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct ModalStyle {
    pub width: f32,
    pub min_height: f32,
    pub max_width: f32,

    pub overlay_color: Color,
    pub surface_color: Color,
    pub border_color: Color,

    pub border_radius: f32,
    pub padding: f32,
}

#[derive(Clone)]
pub struct BasicModalContent {
    pub title: String,
    pub body: String,
    pub dismiss_label: String,
}

impl BasicModalContent {
    pub fn new(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
            dismiss_label: "Close".to_string(),
        }
    }

    pub fn dismiss_label(mut self, label: impl Into<String>) -> Self {
        self.dismiss_label = label.into();
        self
    }
}

impl Default for ModalStyle {
    fn default() -> Self {
        Self {
            width: 480.0,
            min_height: 180.0,
            max_width: 640.0,

            overlay_color: Color::srgba(0.0, 0.0, 0.0, 0.55),
            surface_color: Color::srgba(0.08, 0.08, 0.10, 0.96),
            border_color: Color::srgba(1.0, 1.0, 1.0, 0.10),

            border_radius: 20.0,
            padding: 24.0,
        }
    }
}

const MODAL_SCENE_OFFSET_X: f32 = 180.0;

// ─────────────────────────────────────────────────────────────────────────────
// Modal Commands
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Message, Debug, Clone, Copy)]
pub enum ModalCommand {
    Open(Entity),
    Close(Entity),
    Toggle(Entity),
}

#[derive(Resource, Debug)]
struct ModalDebounceState {
    last_closed_at: HashMap<Entity, f64>,
    reopen_cooldown_secs: f64,
}

impl Default for ModalDebounceState {
    fn default() -> Self {
        Self {
            last_closed_at: HashMap::new(),
            reopen_cooldown_secs: 0.14,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Spawn Modal
// ─────────────────────────────────────────────────────────────────────────────

pub fn spawn_modal(
    commands: &mut Commands,
    parent: Entity,
    modal: Modal,
    style: ModalStyle,
    content: BasicModalContent,
    theme: &ThemeResource,
) -> Entity {
    let colors = theme.current.colors;
    let modal_entity = commands
        .spawn((
            Name::new("modal"),
            modal.clone(),
            crate::components::container::component::Container,
            a11y::TabGroup::modal(),
            (FocusScope { active: modal.open, ..FocusScope::modal() }, a11y::TabIndex(-1)),
            a11y::dialog_node(content.title.clone()),
            ModalReturnFocus::default(),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                padding: UiRect::left(Val::Px(MODAL_SCENE_OFFSET_X)),

                display: if modal.open {
                    Display::Flex
                } else {
                    Display::None
                },

                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,

                ..default()
            },
            ZIndex(1000),
            ModalAnimation::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ))
        .id();
    commands.entity(modal_entity).insert(ChildOf(parent));

    commands.entity(modal_entity).with_children(|root| {
        // ─────────────────────────────────────────────────────────────
        // Backdrop
        // ─────────────────────────────────────────────────────────────

        spawn_backdrop_blur(
            root,
            BackdropBlur::new()
                .opacity(style.overlay_color.alpha())
                .darkness(0.78)
                .intensity(1.0)
                .visible(modal.open),
        );

        root.spawn((
            Button,
            Node {
                position_type: PositionType::Absolute,

                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),

                ..default()
            },
            BackgroundColor(Color::NONE),
            ModalBaseColor(style.overlay_color),
            ModalOverlay,
            Surface::rounded_rect_fill(0.0, Paint::solid(style.overlay_color)),
        ));

        // ─────────────────────────────────────────────────────────────
        // Modal Surface
        // ─────────────────────────────────────────────────────────────

        root.spawn((
            Node {
                width: Val::Px(style.width),
                min_height: Val::Px(style.min_height),
                max_width: Val::Px(style.max_width),

                flex_direction: FlexDirection::Column,
                row_gap: px(18.0),

                padding: UiRect::all(Val::Px(style.padding)),
                border_radius: BorderRadius::all(Val::Px(style.border_radius)),

                ..default()
            },
            BackgroundColor(Color::NONE),
            ModalBaseColor(style.surface_color),
            BorderColor::all(Color::NONE),
            UiTransform::from_scale(Vec2::splat(if modal.open { 1.0 } else { 0.92 })),
            ModalSurface,
            ModalAnimation {
                opacity: if modal.open { 1.0 } else { 0.0 },
                scale: if modal.open { 1.0 } else { 0.92 },
            },
            Surface::rounded_rect_fill(style.border_radius, Paint::solid(style.surface_color))
                .uniform_border(1.0, Paint::solid(style.border_color)),
        ))
        .with_children(|surface| {
            surface
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ModalHeader,
                ))
                .with_children(|header| {
                    header.spawn((
                        ThemedTitle::new(TitleLevel::H2),
                        Text::new(content.title.clone()),
                        TextFont {
                            font_size: FontSize::Px(28.0),
                            ..default()
                        },
                        TextColor(colors.text),
                        Node {
                            flex_grow: 1.0,
                            ..default()
                        },
                    ));

                    header
                        .spawn((
                            Button,
                            ModalCloseButton,
                            a11y::TabIndex(0),
                            a11y::button_node("Close dialog"),
                            ModalActionButton {
                                owner: modal_entity,
                            },
                            Node {
                                width: px(36.0),
                                height: px(36.0),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                border_radius: BorderRadius::all(px(18.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.08)),
                        ))
                        .with_children(|button| {
                            button.spawn((
                                IconNode::new(Icon::feather("x"))
                                    .size(20.0)
                                    .color(Color::WHITE),
                                Node {
                                    width: px(20.0),
                                    height: px(20.0),
                                    ..default()
                                },
                            ));
                        });
                });

            surface
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        flex_grow: 1.0,
                        ..default()
                    },
                    ModalBody,
                ))
                .with_children(|body| {
                    body.spawn((
                        ThemedText::new(TextRole::Body),
                        Text::new(content.body.clone()),
                        TextFont {
                            font_size: FontSize::Px(17.0),
                            ..default()
                        },
                        TextColor(colors.text_muted),
                    ));
                });

            surface
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::FlexEnd,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ModalFooter,
                ))
                .with_children(|footer| {
                    footer
                        .spawn((
                            Button,
                            a11y::TabIndex(0),
                            a11y::button_node(content.dismiss_label.clone()),
                            ModalActionButton {
                                owner: modal_entity,
                            },
                            Node {
                                min_width: px(112.0),
                                height: px(42.0),
                                padding: UiRect::horizontal(px(16.0)),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                border_radius: BorderRadius::all(px(21.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.28, 0.62, 0.94, 0.92)),
                        ))
                        .with_children(|button| {
                            button.spawn((
                                ThemedText::new(TextRole::Label),
                                Text::new(content.dismiss_label.clone()),
                                TextFont {
                                    font_size: FontSize::Px(17.0),
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                });
        });
    });

    modal_entity
}

pub fn spawn_modal_with_surface<F>(
    commands: &mut Commands,
    parent: Entity,
    modal: Modal,
    style: ModalStyle,
    title: impl Into<String>,
    build_surface: F,
    theme: &ThemeResource,
) -> Entity
where
    F: FnOnce(&mut ChildSpawnerCommands),
{
    let title = title.into();
    let colors = theme.current.colors;

    let modal_entity = commands
        .spawn((
            modal.clone(),
            crate::components::container::component::Container,
            a11y::TabGroup::modal(),
            (FocusScope { active: modal.open, ..FocusScope::modal() }, a11y::TabIndex(-1)),
            a11y::dialog_node(title.clone()),
            ModalReturnFocus::default(),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                padding: UiRect::left(Val::Px(MODAL_SCENE_OFFSET_X)),

                display: if modal.open {
                    Display::Flex
                } else {
                    Display::None
                },

                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,

                ..default()
            },
            ZIndex(1000),
            ModalAnimation::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ))
        .id();
    commands.entity(modal_entity).insert(ChildOf(parent));

    commands.entity(modal_entity).with_children(|root| {
        spawn_backdrop_blur(
            root,
            BackdropBlur::new()
                .opacity(style.overlay_color.alpha())
                .darkness(0.78)
                .intensity(1.0)
                .visible(modal.open),
        );

        root.spawn((
            Button,
            Node {
                position_type: PositionType::Absolute,

                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),

                ..default()
            },
            Surface::rounded_rect_fill(0.0, Paint::solid(Color::srgba(0.0, 0.0, 0.0, 0.0))),
            ModalBaseColor(style.overlay_color),
            ModalOverlay,
        ));

        root.spawn((
            Node {
                width: Val::Px(style.width),
                min_height: Val::Px(style.min_height),
                max_width: Val::Px(style.max_width),

                flex_direction: FlexDirection::Column,
                row_gap: px(18.0),

                padding: UiRect::all(Val::Px(style.padding)),
                border_radius: BorderRadius::all(Val::Px(style.border_radius)),

                ..default()
            },
            Surface::rounded_rect_border(
                style.border_radius,
                style.surface_color,
                1.0,
                style.border_color,
            ),
            ModalBaseColor(style.surface_color),
            UiTransform::from_scale(Vec2::splat(if modal.open { 1.0 } else { 0.92 })),
            ModalSurface,
            ModalAnimation {
                opacity: if modal.open { 1.0 } else { 0.0 },
                scale: if modal.open { 1.0 } else { 0.92 },
            },
        ))
        .with_children(|surface| {
            surface
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ModalHeader,
                ))
                .with_children(|header| {
                    header.spawn((
                        ThemedTitle::new(TitleLevel::H2),
                        Text::new(title.clone()),
                        TextFont {
                            font_size: FontSize::Px(28.0),
                            ..default()
                        },
                        TextColor(colors.text),
                        Node {
                            flex_grow: 1.0,
                            ..default()
                        },
                    ));

                    header
                        .spawn((
                            Button,
                            ModalCloseButton,
                            a11y::TabIndex(0),
                            a11y::button_node("Close dialog"),
                            ModalActionButton {
                                owner: modal_entity,
                            },
                            Node {
                                width: px(36.0),
                                height: px(36.0),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                border_radius: BorderRadius::all(px(18.0)),
                                ..default()
                            },
                            Surface::rounded_rect_fill(
                                18.0,
                                Paint::solid(Color::srgba(1.0, 1.0, 1.0, 0.08)),
                            ),
                        ))
                        .with_children(|button| {
                            button.spawn((
                                IconNode::new(Icon::feather("x"))
                                    .size(20.0)
                                    .color(Color::WHITE),
                                Node {
                                    width: px(20.0),
                                    height: px(20.0),
                                    ..default()
                                },
                            ));
                        });
                });

            build_surface(surface);
        });
    });

    modal_entity
}

// ─────────────────────────────────────────────────────────────────────────────
// Open / Close
// ─────────────────────────────────────────────────────────────────────────────

fn modal_commands(
    mut commands: MessageReader<ModalCommand>,
    mut modals: Query<&mut Modal>,
    time: Res<Time>,
    mut debounce: ResMut<ModalDebounceState>,
    mut opened: MessageWriter<ModalOpened>,
    mut closed: MessageWriter<ModalClosed>,
) {
    let mut closed_this_frame = std::collections::HashSet::new();
    let now = time.elapsed_secs_f64();

    for command in commands.read() {
        match *command {
            ModalCommand::Open(entity) => {
                if closed_this_frame.contains(&entity) {
                    continue;
                }

                if let Some(last_closed) = debounce.last_closed_at.get(&entity) {
                    if now - *last_closed < debounce.reopen_cooldown_secs {
                        continue;
                    }
                }

                if let Ok(mut modal) = modals.get_mut(entity) {
                    let was_open = modal.open;
                    modal.open = true;
                    if !was_open {
                        opened.write(ModalOpened { entity });
                    }
                }
            }

            ModalCommand::Close(entity) => {
                if let Ok(mut modal) = modals.get_mut(entity) {
                    let was_open = modal.open;
                    modal.open = false;
                    if was_open {
                        closed.write(ModalClosed { entity });
                    }
                    debounce.last_closed_at.insert(entity, now);
                    // If a close and open are both queued in one frame (rapid click
                    // races), keep the modal closed and defer reopen to the next frame.
                    closed_this_frame.insert(entity);
                }
            }

            ModalCommand::Toggle(entity) => {
                if closed_this_frame.contains(&entity) {
                    continue;
                }

                if let Ok(mut modal) = modals.get_mut(entity) {
                    if modal.open {
                        modal.open = false;
                        closed.write(ModalClosed { entity });
                        debounce.last_closed_at.insert(entity, now);
                        closed_this_frame.insert(entity);
                        continue;
                    }

                    if let Some(last_closed) = debounce.last_closed_at.get(&entity) {
                        if now - *last_closed < debounce.reopen_cooldown_secs {
                            continue;
                        }
                    }

                    modal.open = true;
                    opened.write(ModalOpened { entity });
                    if !modal.open {
                        closed_this_frame.insert(entity);
                    }
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Visibility
// ─────────────────────────────────────────────────────────────────────────────

fn modal_visibility(
    mut modals: Query<(&Modal, &Children, &mut Node)>,
    surfaces: Query<&ModalAnimation, With<ModalSurface>>,
) {
    for (modal, children, mut node) in modals.iter_mut() {
        let mut surface_visible = modal.open;

        if !surface_visible {
            for child in children.iter() {
                if let Ok(animation) = surfaces.get(child) {
                    surface_visible = animation.opacity > 0.02;
                }
            }
        }

        node.display = if surface_visible {
            Display::Flex
        } else {
            Display::None
        };
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Focus management
// ─────────────────────────────────────────────────────────────────────────────

/// Moves keyboard focus into the modal (its close button, or the modal
/// itself as a fallback) when it opens, and restores whatever had focus
/// beforehand once it closes. Combined with `TabGroup::modal()` on the
/// modal root, this traps Tab navigation inside the dialog while it's open.
fn modal_focus_system(
    mut modals: Query<(Entity, &Modal, &mut ModalReturnFocus, &mut FocusScope), Changed<Modal>>,
    close_buttons: Query<(), With<ModalCloseButton>>,
    children_query: Query<&Children>,
    focus: Res<InputFocus>,
    mut focus_requests: MessageWriter<FocusRequest>,
) {
    for (entity, modal, mut return_focus, mut scope) in &mut modals {
        // Closing releases the trap before the shared focus manager validates
        // restoration, even while the closing animation remains visible.
        if scope.active != modal.open {
            scope.active = modal.open;
        }
        if return_focus.was_open == modal.open {
            continue;
        }
        return_focus.was_open = modal.open;
        if modal.open {
            return_focus.previous = focus.get();
            let target = find_descendant(entity, &children_query, &close_buttons).unwrap_or(entity);
            focus_requests.write(FocusRequest {
                target,
                origin: FocusOrigin::Programmatic,
            });
        } else if let Some(previous) = return_focus.previous.take() {
            focus_requests.write(FocusRequest {
                target: previous,
                origin: FocusOrigin::Programmatic,
            });
        }
    }
}

fn find_descendant(
    entity: Entity,
    children_query: &Query<&Children>,
    marker: &Query<(), With<ModalCloseButton>>,
) -> Option<Entity> {
    let children = children_query.get(entity).ok()?;
    for child in children.iter() {
        if marker.contains(child) {
            return Some(child);
        }
        if let Some(found) = find_descendant(child, children_query, marker) {
            return Some(found);
        }
    }
    None
}

// ─────────────────────────────────────────────────────────────────────────────
// Escape
// ─────────────────────────────────────────────────────────────────────────────

fn modal_escape(
    keyboard: Res<ButtonInput<KeyCode>>,
    modals: Query<(Entity, &Modal)>,
    mut commands: MessageWriter<ModalCommand>,
) {
    if !keyboard.just_pressed(KeyCode::Escape) {
        return;
    }

    for (entity, modal) in modals.iter() {
        if modal.open && modal.close_on_escape {
            commands.write(ModalCommand::Close(entity));
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Backdrop
// ─────────────────────────────────────────────────────────────────────────────

fn modal_backdrop(
    mut actions: MessageReader<InteractionActionEvent>,
    overlays: Query<(), With<ModalOverlay>>,
    parents: Query<&ChildOf>,
    modals: Query<&Modal>,
    mut commands: MessageWriter<ModalCommand>,
) {
    for action in actions.read() {
        if action.action != InteractionAction::Activate {
            continue;
        }

        let mut current = Some(action.target);
        let mut overlay_hit = false;
        let mut owner_modal = None;

        while let Some(entity) = current {
            if overlays.contains(entity) {
                overlay_hit = true;
            }
            if modals.get(entity).is_ok() {
                owner_modal = Some(entity);
            }
            current = parents.get(entity).ok().map(ChildOf::parent);
        }

        if !overlay_hit {
            continue;
        }

        let Some(owner_modal) = owner_modal else {
            continue;
        };

        if let Ok(modal) = modals.get(owner_modal) {
            if modal.open && modal.close_on_backdrop {
                commands.write(ModalCommand::Close(owner_modal));
            }
        }
    }
}

fn modal_buttons(
    mut actions: MessageReader<InteractionActionEvent>,
    action_buttons: Query<&ModalActionButton>,
    parents: Query<&ChildOf>,
    modals: Query<&Modal>,
    mut commands: MessageWriter<ModalCommand>,
) {
    for action_event in actions.read() {
        if action_event.action != InteractionAction::Activate {
            continue;
        }

        let mut current = Some(action_event.target);
        let mut owner = None;
        while let Some(entity) = current {
            if let Ok(button) = action_buttons.get(entity) {
                owner = Some(button.owner);
                break;
            }
            current = parents.get(entity).ok().map(ChildOf::parent);
        }

        let Some(owner) = owner else {
            continue;
        };

        if let Ok(modal) = modals.get(owner) {
            if modal.open {
                commands.write(ModalCommand::Close(owner));
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Legacy direct-interaction close button
// ─────────────────────────────────────────────────────────────────────────────

// Some close buttons are clicked without going through the InteractionActionEvent
// pipeline (e.g. headless/unit-test contexts); react to raw Interaction too.
fn modal_close_button_direct_click(
    mut modals: Query<&mut Modal>,
    close_buttons: Query<
        (&Interaction, &ModalActionButton),
        (Changed<Interaction>, With<ModalCloseButton>),
    >,
) {
    for (interaction, button) in &close_buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if let Ok(mut modal) = modals.get_mut(button.owner) {
            if modal.open {
                modal.open = false;
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Animation
// ─────────────────────────────────────────────────────────────────────────────
fn modal_animation(
    time: Res<Time>,
    policy: Res<AccessibilityVisualPolicyResource>,

    mut modals: Query<(&Modal, &Children), With<Modal>>,

    mut surfaces: Query<
        (
            Option<&ModalOverlay>,
            Option<&ModalSurface>,
            Option<&mut ModalAnimation>,
            &ModalBaseColor,
            &mut Surface,
            Option<&mut UiTransform>,
        ),
    >,
    mut backdrops: Query<&mut BackdropBlur>,
) {
    // Like shared decorative animation tracks, reduced motion snaps to the
    // endpoint, including when the policy changes during a transition.
    let interpolate = |current: f32, target: f32, rate: f32| {
        if policy.current.reduced_motion {
            target
        } else {
            current + (target - current) * (1.0 - (-rate * time.delta_secs()).exp())
        }
    };
    for (modal, children) in modals.iter_mut() {
        for child in children.iter() {
            if let Ok(mut backdrop) = backdrops.get_mut(child) {
                if backdrop.visible != modal.open {
                    backdrop.visible = modal.open;
                }
            }

            let Ok((overlay_marker, surface_marker, animation, base_color, mut surface, transform)) =
                surfaces.get_mut(child)
            else {
                continue;
            };

            if overlay_marker.is_some() {
                let overlay_target = if modal.open { base_color.0.alpha() } else { 0.0 };
                let current = match &surface.fill {
                    Paint::Solid(color) => color.alpha(),
                    _ => 0.0,
                };
                // Both current and target are absolute alpha, not opacity
                // factors: multiplying by base alpha again prevents convergence.
                let next = interpolate(current, overlay_target, 12.0);
                let fill = Paint::solid(base_color.0.with_alpha(next));
                if surface.fill != fill {
                    surface.fill = fill;
                }
                continue;
            }

            let (Some(mut animation), Some(mut transform)) = (animation, transform) else {
                continue;
            };

            if surface_marker.is_none() {
                continue;
            }

            let target_opacity = if modal.open { 1.0 } else { 0.0 };
            let target_scale = if modal.open { 1.0 } else { 0.92 };

            let opacity = interpolate(animation.opacity, target_opacity, 14.0);
            let scale = interpolate(animation.scale, target_scale, 16.0);
            if animation.opacity != opacity {
                animation.opacity = opacity;
            }
            if animation.scale != scale {
                animation.scale = scale;
            }

            let fill = Paint::solid(base_color.0.with_alpha(base_color.0.alpha() * opacity));
            if surface.fill != fill {
                surface.fill = fill;
            }
            let scale = Vec2::splat(scale);
            if transform.scale != scale {
                transform.scale = scale;
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Plugin
// ─────────────────────────────────────────────────────────────────────────────

pub struct ModalPlugin;

impl Plugin for ModalPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AccessibilityVisualPolicyResource>()
            .init_resource::<ModalDebounceState>()
            .add_message::<ModalOpened>()
            .add_message::<ModalClosed>()
            .add_message::<ModalCommand>()
            .add_systems(
                Update,
                (
                    modal_escape,
                    modal_backdrop,
                    modal_buttons,
                    modal_close_button_direct_click,
                    modal_commands,
                    modal_animation,
                    modal_visibility,
                ).chain(),
            )
            .add_systems(
                PostUpdate,
                modal_focus_system.before(FocusSystems::ApplyRequests),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::focus::{FocusPlugin, FocusRejectReason, FocusRejected};

    fn modal_app() -> App {
        let mut app = App::new();
        app.init_resource::<Time>()
            .init_resource::<InputFocus>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .add_message::<InteractionActionEvent>()
            .add_plugins((FocusPlugin, ModalPlugin));
        app
    }

    fn mount(app: &mut App, open: bool) -> (Entity, Entity) {
        let modal = app.world_mut().spawn((
            Modal { open, ..default() },
            ModalReturnFocus::default(),
            FocusScope { active: open, ..FocusScope::modal() },
            Node::default(),
            // No renderer/visibility propagation runs in this headless app.
            InheritedVisibility::VISIBLE,
            a11y::TabIndex(-1),
        )).id();
        let close = app.world_mut().spawn((
            Button,
            ModalCloseButton,
            ModalActionButton { owner: modal },
            a11y::TabIndex(0),
            InheritedVisibility::VISIBLE,
            ChildOf(modal),
        )).id();
        (modal, close)
    }

    fn request(app: &mut App, target: Entity) {
        app.world_mut().write_message(FocusRequest { target, origin: FocusOrigin::Programmatic });
    }

    fn mount_visual_modal(app: &mut App, custom: bool) -> (Entity, Entity, Entity, Entity) {
        // Use the real spawn helpers and composed focus/modal schedules, without
        // a renderer. Explicit policy makes these tests independent of the env.
        app.world_mut().insert_resource(AccessibilityVisualPolicyResource {
            current: crate::theme::AccessibilityVisualPolicy::default(),
        });
        let parent = app.world_mut().spawn(Node::default()).id();
        let theme = ThemeResource::default();
        let mut commands = app.world_mut().commands();
        let modal = if custom {
            spawn_modal_with_surface(&mut commands, parent, Modal::new(), ModalStyle::default(),
                "Title", |_| {}, &theme)
        } else {
            spawn_modal(&mut commands, parent, Modal::new(), ModalStyle::default(),
                BasicModalContent::new("Title", "Body"), &theme)
        };
        app.world_mut().flush();
        let children = app.world().get::<Children>(modal).unwrap();
        let surface = children.iter().find(|&e| app.world().get::<ModalSurface>(e).is_some()).unwrap();
        let overlay = children.iter().find(|&e| app.world().get::<ModalOverlay>(e).is_some()).unwrap();
        let backdrop = children.iter().find(|&e| app.world().get::<BackdropBlur>(e).is_some()).unwrap();
        (modal, surface, overlay, backdrop)
    }

    fn advance_modal(app: &mut App, seconds: f32) {
        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_secs_f32(seconds));
        app.update();
    }

    fn alpha(app: &App, entity: Entity) -> f32 {
        match &app.world().get::<Surface>(entity).unwrap().fill {
            Paint::Solid(color) => color.alpha(),
            _ => panic!("expected solid modal fill"),
        }
    }

    #[test]
    fn composed_modal_animates_ui_transform_and_preserves_close_timing() {
        for custom in [false, true] {
            let mut app = modal_app();
            let (modal, surface, _, _) = mount_visual_modal(&mut app, custom);
            assert_eq!(app.world().get::<UiTransform>(surface).unwrap().scale, Vec2::splat(0.92));
            assert!(app.world().get::<Transform>(surface).is_none());
            app.world_mut().write_message(ModalCommand::Open(modal));
            advance_modal(&mut app, 1.0 / 60.0);
            let animation = app.world().get::<ModalAnimation>(surface).unwrap();
            let expected = 0.92 + 0.08 * (1.0 - (-16.0_f32 / 60.0).exp());
            assert!((animation.scale - expected).abs() < 1e-6);
            assert_eq!(app.world().get::<UiTransform>(surface).unwrap().scale, Vec2::splat(animation.scale));
            assert!(animation.opacity > 0.0 && animation.opacity < 1.0);
            assert_eq!(app.world().get::<Node>(modal).unwrap().display, Display::Flex);

            app.world_mut().write_message(ModalCommand::Close(modal));
            advance_modal(&mut app, 1.0 / 60.0);
            // Focus is released immediately, but the legacy opacity threshold
            // still keeps the closing surface displayed until its fade finishes.
            assert!(!app.world().get::<FocusScope>(modal).unwrap().active);
            assert_eq!(app.world().get::<Node>(modal).unwrap().display, Display::Flex);
            for _ in 0..120 {
                advance_modal(&mut app, 1.0 / 60.0);
            }
            assert_eq!(app.world().get::<Node>(modal).unwrap().display, Display::None);
            assert!((app.world().get::<UiTransform>(surface).unwrap().scale.x - 0.92).abs() < 1e-6);
        }
    }

    #[test]
    fn composed_modal_overlay_converges_to_absolute_base_alpha() {
        for custom in [false, true] {
            let mut app = modal_app();
            let (modal, _, overlay, _) = mount_visual_modal(&mut app, custom);
            let base = app.world().get::<ModalBaseColor>(overlay).unwrap().0;
            app.world_mut().get_mut::<Surface>(overlay).unwrap().fill = Paint::solid(base.with_alpha(0.2));
            app.world_mut().write_message(ModalCommand::Open(modal));
            advance_modal(&mut app, 1.0 / 60.0);
            let expected = 0.2 + (base.alpha() - 0.2) * (1.0 - (-12.0_f32 / 60.0).exp());
            assert!((alpha(&app, overlay) - expected).abs() < 1e-6);
            for _ in 0..120 {
                let previous = alpha(&app, overlay);
                advance_modal(&mut app, 1.0 / 60.0);
                assert!(alpha(&app, overlay) >= previous);
                assert!(alpha(&app, overlay) <= base.alpha());
            }
            assert!((alpha(&app, overlay) - base.alpha()).abs() < 1e-6);
            app.world_mut().write_message(ModalCommand::Close(modal));
            for _ in 0..120 {
                let previous = alpha(&app, overlay);
                advance_modal(&mut app, 1.0 / 60.0);
                assert!(alpha(&app, overlay) <= previous);
            }
            assert!(alpha(&app, overlay) < 1e-6);
        }
    }

    #[test]
    fn composed_modal_reduced_motion_snaps_and_idle_visuals_stay_clean() {
        for custom in [false, true] {
            let mut app = modal_app();
            let (modal, surface, overlay, backdrop) = mount_visual_modal(&mut app, custom);
            app.world_mut().write_message(ModalCommand::Open(modal));
            advance_modal(&mut app, 1.0 / 60.0);
            assert!(app.world().get::<ModalAnimation>(surface).unwrap().scale < 1.0);
            app.world_mut().resource_mut::<AccessibilityVisualPolicyResource>().current.reduced_motion = true;
            // A policy change snaps an in-flight transition even at zero delta.
            advance_modal(&mut app, 0.0);
            for open in [true, false] {
                if !open {
                    app.world_mut().write_message(ModalCommand::Close(modal));
                    advance_modal(&mut app, 0.0);
                }
                let animation = app.world().get::<ModalAnimation>(surface).unwrap();
                assert_eq!(animation.opacity, if open { 1.0 } else { 0.0 });
                assert_eq!(animation.scale, if open { 1.0 } else { 0.92 });
                assert_eq!(app.world().get::<UiTransform>(surface).unwrap().scale, Vec2::splat(animation.scale));
                for entity in [surface, overlay] {
                    let base = app.world().get::<ModalBaseColor>(entity).unwrap().0;
                    assert_eq!(alpha(&app, entity), if open { base.alpha() } else { 0.0 });
                }
                assert_eq!(app.world().get::<BackdropBlur>(backdrop).unwrap().visible, open);
                assert_eq!(app.world().get::<Node>(modal).unwrap().display, if open { Display::Flex } else { Display::None });

                app.world_mut().clear_trackers();
                advance_modal(&mut app, 1.0 / 60.0);
                assert!(!app.world().entity(surface).get_ref::<ModalAnimation>().unwrap().is_changed());
                assert!(!app.world().entity(surface).get_ref::<UiTransform>().unwrap().is_changed());
                for entity in [surface, overlay] {
                    assert!(!app.world().entity(entity).get_ref::<Surface>().unwrap().is_changed());
                }
                assert!(!app.world().entity(backdrop).get_ref::<BackdropBlur>().unwrap().is_changed());
            }
        }
    }

    #[test]
    fn close_command_wins_over_same_frame_reopen() {
        let mut app = modal_app();
        let (modal, _) = mount(&mut app, true);

        app.world_mut().write_message(ModalCommand::Close(modal));
        app.world_mut().write_message(ModalCommand::Open(modal));
        app.update();

        assert!(!app.world().get::<Modal>(modal).unwrap().open);
    }

    #[test]
    fn close_then_immediate_reopen_is_debounced() {
        let mut app = modal_app();
        let (modal, _) = mount(&mut app, true);

        app.world_mut().write_message(ModalCommand::Close(modal));
        app.update();
        assert!(!app.world().get::<Modal>(modal).unwrap().open);

        // Next-frame reopen inside cooldown should be ignored.
        app.world_mut().write_message(ModalCommand::Open(modal));
        advance_modal(&mut app, 1.0 / 60.0);
        assert!(!app.world().get::<Modal>(modal).unwrap().open);

        // Reopen once cooldown has elapsed.
        advance_modal(&mut app, 0.2);
        app.world_mut().write_message(ModalCommand::Open(modal));
        app.update();
        assert!(app.world().get::<Modal>(modal).unwrap().open);
    }

    #[test]
    fn mounted_modal_traps_only_while_open_and_restores_in_closing_frame() {
        let mut app = modal_app();
        let outside = app.world_mut().spawn(a11y::TabIndex(0)).id();
        let (modal, close) = mount(&mut app, false);
        request(&mut app, outside);
        app.update();
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(outside));
        assert!(!app.world().get::<FocusScope>(modal).unwrap().active);

        app.world_mut().write_message(ModalCommand::Open(modal));
        app.update();
        assert!(app.world().get::<FocusScope>(modal).unwrap().active);
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(close));

        request(&mut app, outside);
        app.update();
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(close));
        let rejected: Vec<_> = app.world_mut().resource_mut::<Messages<FocusRejected>>().drain().collect();
        assert_eq!(rejected.len(), 1);
        assert_eq!(rejected[0].reason, FocusRejectReason::OutsideActiveScope);

        // Reissuing Open or changing unrelated options must not replace the
        // original return target with a descendant of the modal itself.
        app.world_mut().write_message(ModalCommand::Open(modal));
        app.world_mut().get_mut::<Modal>(modal).unwrap().close_on_escape = false;
        app.update();
        app.world_mut().write_message(ModalCommand::Close(modal));
        app.update();
        assert!(!app.world().get::<FocusScope>(modal).unwrap().active);
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(outside));
        app.update();
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(outside));

        // Direct state changes and the legacy close-button path also complete
        // focus changes without waiting for another PreUpdate.
        app.world_mut().get_mut::<Modal>(modal).unwrap().open = true;
        app.update();
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(close));
        app.world_mut().get_mut::<Interaction>(close).unwrap().set_if_neq(Interaction::Pressed);
        app.update();
        assert!(!app.world().get::<FocusScope>(modal).unwrap().active);
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(outside));
    }

    #[test]
    fn initially_open_modal_restores_and_closed_sibling_does_not_trap() {
        let mut app = modal_app();
        let outside = app.world_mut().spawn(a11y::TabIndex(0)).id();
        app.world_mut().insert_resource(InputFocus::from_entity(outside));
        let (closed, _) = mount(&mut app, false);
        let (modal, close) = mount(&mut app, true);
        app.update();
        assert!(!app.world().get::<FocusScope>(closed).unwrap().active);
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(close));
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Escape);
        app.update();
        assert!(!app.world().get::<FocusScope>(modal).unwrap().active);
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(outside));
    }

    #[test]
    fn spawn_helpers_initialize_closed_scopes_inactive() {
        let mut app = modal_app();
        let parent = app.world_mut().spawn(Node::default()).id();
        let theme = ThemeResource::default();
        let mut commands = app.world_mut().commands();
        let basic = spawn_modal(&mut commands, parent, Modal::new(), ModalStyle::default(),
            BasicModalContent::new("Title", "Body"), &theme);
        let custom = spawn_modal_with_surface(&mut commands, parent, Modal::new(), ModalStyle::default(),
            "Title", |_| {}, &theme);
        app.world_mut().flush();
        for modal in [basic, custom] {
            assert!(!app.world().get::<FocusScope>(modal).unwrap().active);
            assert_eq!(app.world().get::<a11y::TabIndex>(modal).unwrap().0, -1);
        }
    }
}
