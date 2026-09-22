use bevy::a11y::AccessibilityNode;
use bevy::input::{ButtonState, keyboard::KeyboardInput};
use bevy::input_focus::InputFocus;
use bevy::prelude::*;

use crate::rendering::{Border, Paint, Surface};
use crate::primitives::a11y;
use crate::icons::{Icon, IconNode};
use crate::components::text::{TextRole, ThemedText};
use crate::theme::{ThemeResource, dark_theme};

// ============================================================
// DROPDOWN
// ============================================================

#[derive(Component)]
pub struct Dropdown {
    pub placeholder: String,
    pub options: Vec<DropdownOption>,
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct DropdownState {
    pub selected: Option<usize>,
    pub highlighted: Option<usize>,
    pub open: bool,
}

#[derive(Clone, Debug)]
pub struct DropdownOption {
    pub label: String,
    pub value: String,
}

impl DropdownOption {
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
        }
    }
}

// ============================================================
// CONFIGURATION
// ============================================================

pub struct DropdownConfig {
    pub placeholder: String,
    pub options: Vec<DropdownOption>,
    pub selected: Option<usize>,
    pub open_animation: DropdownMenuAnimationKind,
    pub close_animation: DropdownMenuAnimationKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DropdownMenuAnimationKind {
    Fade,
    FallFromTrigger,
    ScaleInOut,
    SlideFromLeft,
    SlideFromRight,
}

impl DropdownConfig {
    pub fn new(placeholder: impl Into<String>) -> Self {
        Self {
            placeholder: placeholder.into(),
            options: Vec::new(),
            selected: None,
            open_animation: DropdownMenuAnimationKind::Fade,
            close_animation: DropdownMenuAnimationKind::Fade,
        }
    }

    pub fn option(mut self, label: impl Into<String>, value: impl Into<String>) -> Self {
        self.options.push(DropdownOption::new(label, value));
        self
    }

    pub fn selected(mut self, index: usize) -> Self {
        self.selected = Some(index);
        self
    }

    pub fn open_animation(mut self, animation: DropdownMenuAnimationKind) -> Self {
        self.open_animation = animation;
        self
    }

    pub fn close_animation(mut self, animation: DropdownMenuAnimationKind) -> Self {
        self.close_animation = animation;
        self
    }
}

// ============================================================
// COMPONENT MARKERS
// ============================================================

#[derive(Component, Clone, Copy)]
pub struct DropdownSurface {
    pub owner: Entity,
}

#[derive(Component, Clone, Copy)]
pub struct DropdownLabel {
    pub owner: Entity,
}

#[derive(Component, Clone, Copy)]
pub struct DropdownArrow {
    pub owner: Entity,
}

#[derive(Component, Clone, Copy)]
pub struct DropdownMenu {
    pub owner: Entity,
}

#[derive(Component, Clone, Copy)]
pub struct DropdownOptionButton {
    pub owner: Entity,
    pub index: usize,
}

#[derive(Component, Clone, Copy)]
pub struct DropdownOptionLabel {
    pub owner: Entity,
    pub index: usize,
}

#[derive(Component)]
struct DropdownMenuAnimation {
    progress: f32,
    target: f32,
    velocity: f32,
    stiffness: f32,
    damping: f32,
}

#[derive(Component, Clone, Copy)]
struct DropdownAnimationProfile {
    open: DropdownMenuAnimationKind,
    close: DropdownMenuAnimationKind,
}

#[derive(Clone, Copy)]
struct DropdownMenuVisual {
    top_px: f32,
    left_percent: f32,
    width_percent: f32,
    opacity: f32,
}

#[derive(Resource, Default)]
struct DropdownFocus {
    entity: Option<Entity>,
}

// ============================================================
// EVENTS / HOOKS
// ============================================================

#[derive(Message, Debug, Clone)]
pub enum DropdownEvent {
    Opened {
        entity: Entity,
    },
    Closed {
        entity: Entity,
    },
    Changed {
        entity: Entity,
        index: usize,
        value: String,
    },
}

// ============================================================
// SPAWN
// ============================================================

pub fn spawn_dropdown(parent: &mut ChildSpawnerCommands, config: DropdownConfig) -> Entity {
    let colors = dark_theme().colors;

    let DropdownConfig {
        placeholder,
        options,
        selected,
        open_animation,
        close_animation,
    } = config;

    let clamped_selected = selected.filter(|index| *index < options.len());
    let accessible_label = placeholder.clone();

    let mut dropdown_entity = parent.spawn((
        Dropdown {
            placeholder,
            options: options.clone(),
        },
        DropdownState {
            selected: clamped_selected,
            highlighted: clamped_selected,
            open: false,
        },
        DropdownAnimationProfile {
            open: open_animation,
            close: close_animation,
        },
        Node {
            width: percent(100),
            height: px(38),
            position_type: PositionType::Relative,
            overflow: Overflow::visible(),
            ..default()
        },
    ));

    let dropdown_id = dropdown_entity.id();

    dropdown_entity.with_children(|dropdown| {
        dropdown
            .spawn((
                Button,
                DropdownSurface { owner: dropdown_id },
                a11y::TabIndex(0),
                a11y::combo_box_node(accessible_label, false),
                Node {
                    width: percent(100),
                    height: px(38),
                    align_items: AlignItems::Center,
                    padding: UiRect::horizontal(px(12)),
                    justify_content: JustifyContent::SpaceBetween,
                    border_radius: BorderRadius::all(px(10.0)),
                    ..default()
                },
                BackgroundColor(Color::NONE),
                Surface::rounded_rect_fill(10.0, Paint::solid(colors.surface))
                    .uniform_border(1.0, Paint::solid(colors.border.with_alpha(0.65))),
            ))
            .with_children(|surface| {
                surface.spawn((
                    DropdownLabel { owner: dropdown_id },
                    ThemedText::new(TextRole::Body),
                    Text::new(""),
                    TextFont {
                        font_size: FontSize::Px(14.0),
                        ..default()
                    },
                    TextColor(colors.text),
                ));

                surface.spawn((
                    DropdownArrow { owner: dropdown_id },
                    IconNode::new(Icon::feather("chevron-down"))
                        .size(16.0)
                        .color(colors.text_muted),
                    Node {
                        width: px(16.0),
                        height: px(16.0),
                        ..default()
                    },
                ));
            });

        dropdown
            .spawn((
                DropdownMenu { owner: dropdown_id },
                DropdownMenuAnimation {
                    progress: 0.0,
                    target: 0.0,
                    velocity: 0.0,
                    stiffness: 220.0,
                    damping: 26.0,
                },
                Node {
                    width: percent(100),
                    position_type: PositionType::Absolute,
                    top: px(42),
                    left: px(0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(px(6)),
                    row_gap: px(4),
                    border_radius: BorderRadius::all(px(10.0)),
                    ..default()
                },
                Visibility::Hidden,
                BackgroundColor(Color::NONE),
                Surface::rounded_rect_fill(
                    10.0,
                    Paint::solid(colors.surface_elevated.with_alpha(0.0)),
                )
                .uniform_border(1.0, Paint::solid(colors.border.with_alpha(0.55))),
                GlobalZIndex(200),
            ))
            .with_children(|menu| {
                for (index, option) in options.iter().enumerate() {
                    menu.spawn((
                        Button,
                        DropdownOptionButton {
                            owner: dropdown_id,
                            index,
                        },
                        a11y::list_box_option_node(
                            option.label.clone(),
                            clamped_selected == Some(index),
                        ),
                        Node {
                            width: percent(100),
                            height: px(34),
                            align_items: AlignItems::Center,
                            padding: UiRect::horizontal(px(10)),
                            border_radius: BorderRadius::all(px(8.0)),
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                        Surface::rounded_rect_fill(8.0, Paint::solid(colors.surface.with_alpha(0.0))),
                    ))
                    .with_children(|option_button| {
                        option_button.spawn((
                            DropdownOptionLabel {
                                owner: dropdown_id,
                                index,
                            },
                            ThemedText::new(TextRole::Label),
                            Text::new(option.label.clone()),
                            TextFont {
                                font_size: FontSize::Px(13.0),
                                ..default()
                            },
                            TextColor(colors.text.with_alpha(0.0)),
                        ));
                    });
                }
            });
    });

    dropdown_id
}

// ============================================================
// INTERACTION
// ============================================================

fn dropdown_surface_system(
    mut surface_query: Query<
        (&Interaction, &DropdownSurface),
        (Changed<Interaction>, With<Button>),
    >,
    mut dropdown_query: Query<(&Dropdown, &mut DropdownState)>,
    mut focus: ResMut<DropdownFocus>,
    mut events: MessageWriter<DropdownEvent>,
) {
    for (interaction, surface) in &mut surface_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let Ok((dropdown, mut state)) = dropdown_query.get_mut(surface.owner) else {
            continue;
        };

        focus.entity = Some(surface.owner);

        state.open = !state.open;
        if state.open {
            state.highlighted = state
                .selected
                .or_else(|| (!dropdown.options.is_empty()).then_some(0));
        } else {
            state.highlighted = state.selected;
        }

        if state.open {
            events.write(DropdownEvent::Opened {
                entity: surface.owner,
            });
        } else {
            events.write(DropdownEvent::Closed {
                entity: surface.owner,
            });
        }
    }
}

fn dropdown_option_system(
    option_query: Query<
        (&Interaction, &DropdownOptionButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut dropdown_query: Query<(&Dropdown, &mut DropdownState)>,
    mut focus: ResMut<DropdownFocus>,
    mut events: MessageWriter<DropdownEvent>,
) {
    for (interaction, option_button) in &option_query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let Ok((dropdown, mut state)) = dropdown_query.get_mut(option_button.owner) else {
            continue;
        };

        if option_button.index >= dropdown.options.len() {
            continue;
        }

        focus.entity = Some(option_button.owner);
        state.selected = Some(option_button.index);
        state.highlighted = Some(option_button.index);

        if state.open {
            state.open = false;
            events.write(DropdownEvent::Closed {
                entity: option_button.owner,
            });
        }

        let selected_option = &dropdown.options[option_button.index];
        events.write(DropdownEvent::Changed {
            entity: option_button.owner,
            index: option_button.index,
            value: selected_option.value.clone(),
        });
    }
}

/// Bridges Tab-key focus into this widget's own `DropdownFocus` tracking, so
/// tabbing to the trigger (without clicking) also unlocks the existing
/// arrow-key / Enter / Escape keyboard navigation below.
fn dropdown_keyboard_focus_bridge_system(
    focus: Res<InputFocus>,
    surfaces: Query<&DropdownSurface>,
    mut dropdown_focus: ResMut<DropdownFocus>,
) {
    if !focus.is_changed() {
        return;
    }
    if let Some(surface) = focus.get().and_then(|entity| surfaces.get(entity).ok()) {
        dropdown_focus.entity = Some(surface.owner);
    }
}

fn dropdown_click_outside_close_system(
    mouse: Res<ButtonInput<MouseButton>>,
    surface_press_query: Query<&Interaction, (Changed<Interaction>, With<DropdownSurface>)>,
    option_press_query: Query<&Interaction, (Changed<Interaction>, With<DropdownOptionButton>)>,
    mut dropdown_query: Query<(Entity, &mut DropdownState)>,
    mut focus: ResMut<DropdownFocus>,
    mut events: MessageWriter<DropdownEvent>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let pressed_dropdown_surface = surface_press_query
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed);
    let pressed_dropdown_option = option_press_query
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed);

    if pressed_dropdown_surface || pressed_dropdown_option {
        return;
    }

    focus.entity = None;

    for (entity, mut state) in &mut dropdown_query {
        if !state.open {
            continue;
        }

        state.open = false;
        state.highlighted = state.selected;
        events.write(DropdownEvent::Closed { entity });
    }
}

fn dropdown_keyboard_navigation_system(
    mut keyboard: MessageReader<KeyboardInput>,
    focus: Res<DropdownFocus>,
    mut dropdown_query: Query<(&Dropdown, &mut DropdownState)>,
    mut events: MessageWriter<DropdownEvent>,
) {
    let Some(focused_entity) = focus.entity else {
        return;
    };

    let Ok((dropdown, mut state)) = dropdown_query.get_mut(focused_entity) else {
        return;
    };

    let option_count = dropdown.options.len();
    if option_count == 0 {
        return;
    }

    for event in keyboard.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }

        match event.key_code {
            KeyCode::ArrowDown => {
                if !state.open {
                    state.open = true;
                    state.highlighted = state.selected.or(Some(0));
                    events.write(DropdownEvent::Opened {
                        entity: focused_entity,
                    });
                } else {
                    let next = match state.highlighted {
                        Some(current) => (current + 1) % option_count,
                        None => 0,
                    };
                    state.highlighted = Some(next);
                }
            }

            KeyCode::ArrowUp => {
                if !state.open {
                    state.open = true;
                    state.highlighted = state.selected.or(Some(option_count - 1));
                    events.write(DropdownEvent::Opened {
                        entity: focused_entity,
                    });
                } else {
                    let next = match state.highlighted {
                        Some(current) => {
                            if current == 0 {
                                option_count - 1
                            } else {
                                current - 1
                            }
                        }
                        None => option_count - 1,
                    };
                    state.highlighted = Some(next);
                }
            }

            KeyCode::Enter | KeyCode::Space => {
                if !state.open {
                    state.open = true;
                    state.highlighted = state.selected.or(Some(0));
                    events.write(DropdownEvent::Opened {
                        entity: focused_entity,
                    });
                    continue;
                }

                if let Some(index) = state.highlighted.filter(|idx| *idx < option_count) {
                    state.selected = Some(index);

                    let option = &dropdown.options[index];
                    events.write(DropdownEvent::Changed {
                        entity: focused_entity,
                        index,
                        value: option.value.clone(),
                    });
                }

                state.open = false;
                state.highlighted = state.selected;
                events.write(DropdownEvent::Closed {
                    entity: focused_entity,
                });
            }

            KeyCode::Escape => {
                if !state.open {
                    continue;
                }

                state.open = false;
                state.highlighted = state.selected;
                events.write(DropdownEvent::Closed {
                    entity: focused_entity,
                });
            }

            _ => {}
        }
    }
}

// ============================================================
// VISUAL STATE + ANIMATION
// ============================================================

fn dropdown_visual_system(
    theme: Res<ThemeResource>,
    dropdown_query: Query<(&Dropdown, &DropdownState)>,
    mut surface_query: Query<
        (
            Option<&DropdownSurface>,
            Option<&DropdownMenu>,
            &mut Surface,
        ),
        Or<(With<DropdownSurface>, With<DropdownMenu>)>,
    >,
    mut label_query: Query<(&DropdownLabel, &mut Text), Without<DropdownArrow>>,
    mut arrow_query: Query<(&DropdownArrow, &mut IconNode), Without<DropdownLabel>>,
    mut label_color_query: Query<(&DropdownLabel, &mut TextColor), Without<DropdownArrow>>,
) {
    let colors = theme.current.colors;
    let is_dark = theme.current.mode == crate::theme::ThemeMode::Dark;

    for (marker, mut label) in &mut label_query {
        let Ok((dropdown, state)) = dropdown_query.get(marker.owner) else {
            continue;
        };

        let label_text = state
            .selected
            .and_then(|index| dropdown.options.get(index))
            .map(|option| option.label.as_str())
            .unwrap_or(dropdown.placeholder.as_str());

        *label = Text::new(label_text);
    }

    for (surface_marker, menu_marker, mut background) in &mut surface_query {
        let owner = surface_marker
            .map(|marker| marker.owner)
            .or_else(|| menu_marker.map(|marker| marker.owner));

        let Some(owner) = owner else {
            continue;
        };

        if dropdown_query.get(owner).is_err() {
            continue;
        }

        if surface_marker.is_some() {
            background.fill = Paint::solid(if is_dark {
                colors.surface.with_alpha(0.94)
            } else {
                colors.surface
            });
            background.border = Some(Border::new(
                1.0,
                Paint::solid(colors.border.with_alpha(if is_dark { 0.92 } else { 0.60 })),
            ));
        } else if menu_marker.is_some() {
            let alpha = match &background.fill {
                Paint::Solid(color) => color.alpha(),
                _ => 1.0,
            };
            background.fill = Paint::solid(colors.surface_elevated.with_alpha(alpha));
            background.border = Some(Border::new(
                1.0,
                Paint::solid(colors.border.with_alpha(if is_dark { 0.90 } else { 0.55 })),
            ));
        }
    }

    for (marker, mut color) in &mut label_color_query {
        let Ok((dropdown, state)) = dropdown_query.get(marker.owner) else {
            continue;
        };

        let is_placeholder = state
            .selected
            .and_then(|index| dropdown.options.get(index))
            .is_none();

        color.0 = if is_placeholder {
            colors.text_muted
        } else {
            colors.text
        };
    }

    for (marker, mut arrow) in &mut arrow_query {
        let Ok((_, state)) = dropdown_query.get(marker.owner) else {
            continue;
        };

        arrow.icon = if state.open {
            Icon::feather("chevron-up")
        } else {
            Icon::feather("chevron-down")
        };

        arrow.color = if state.open {
            colors.text
        } else {
            colors.text_muted
        };
    }
}

fn dropdown_menu_animation_system(
    time: Res<Time>,
    theme: Res<ThemeResource>,
    dropdown_meta_query: Query<(&DropdownState, &DropdownAnimationProfile)>,
    mut menu_query: Query<(
        &DropdownMenu,
        &mut DropdownMenuAnimation,
        &mut Node,
        &mut Visibility,
        &mut Surface,
    )>,
) {
    let colors = theme.current.colors;

    for (menu, mut animation, mut node, mut visibility, mut background) in &mut menu_query {
        let Ok((state, profile)) = dropdown_meta_query.get(menu.owner) else {
            continue;
        };

        animation.target = if state.open { 1.0 } else { 0.0 };
        spring_step(&mut animation, time.delta_secs());
        let clamped_progress = animation.progress.clamp(0.0, 1.0);
        let active_animation = if state.open { profile.open } else { profile.close };
        let visual = active_animation.sample(clamped_progress);

        if animation.target > 0.0 || clamped_progress > 0.001 {
            *visibility = Visibility::Inherited;
        } else {
            *visibility = Visibility::Hidden;
        }

        node.top = px(visual.top_px);
        node.left = percent(visual.left_percent);
        node.width = percent(visual.width_percent);
        background.fill = Paint::solid(colors.surface_elevated.with_alpha(0.98 * visual.opacity));
    }
}

fn dropdown_option_visual_system(
    theme: Res<ThemeResource>,
    dropdown_query: Query<&DropdownState>,
    menu_query: Query<(&DropdownMenu, &DropdownMenuAnimation)>,
    mut option_query: Query<(&DropdownOptionButton, &Interaction, &mut Surface)>,
    mut option_label_query: Query<(&DropdownOptionLabel, &mut TextColor)>,
) {
    let colors = theme.current.colors;
    let is_dark = theme.current.mode == crate::theme::ThemeMode::Dark;

    for (option, interaction, mut background) in &mut option_query {
        let Ok(state) = dropdown_query.get(option.owner) else {
            continue;
        };

        let menu_progress = menu_query
            .iter()
            .find_map(|(menu, animation)| {
                (menu.owner == option.owner).then_some(animation.progress)
            })
            .unwrap_or(0.0);

        let is_active = if state.open {
            state.highlighted == Some(option.index)
        } else {
            state.selected == Some(option.index)
        };

        let base_color = if is_active {
            colors.primary
        } else if *interaction == Interaction::Hovered {
            if is_dark {
                colors.surface_elevated
            } else {
                colors.secondary
            }
        } else {
            colors.surface
        };

        let alpha = smoothstep(menu_progress);
        let max_alpha = if is_active { 0.90 } else { 0.78 };
        background.fill = Paint::solid(base_color.with_alpha(alpha * max_alpha));
    }

    for (label, mut color) in &mut option_label_query {
        let Ok(state) = dropdown_query.get(label.owner) else {
            continue;
        };

        let menu_progress = menu_query
            .iter()
            .find_map(|(menu, animation)| (menu.owner == label.owner).then_some(animation.progress))
            .unwrap_or(0.0);

        let text_alpha = smoothstep(menu_progress);
        let selected_tint = if state.open {
            state.highlighted == Some(label.index)
        } else {
            state.selected == Some(label.index)
        };

        color.0 = if selected_tint {
            colors.surface.with_alpha(text_alpha)
        } else {
            colors.text.with_alpha(text_alpha * 0.95)
        };
    }
}

fn smoothstep(t: f32) -> f32 {
    let x = t.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

impl DropdownMenuAnimationKind {
    fn sample(self, progress: f32) -> DropdownMenuVisual {
        let eased = smoothstep(progress);

        match self {
            Self::Fade => DropdownMenuVisual {
                top_px: 42.0,
                left_percent: 0.0,
                width_percent: 100.0,
                opacity: eased,
            },

            // Starts above the button seam and settles into place.
            // On close, the same curve runs backward and scrolls up.
            Self::FallFromTrigger => {
                let travel_px = 14.0;

                DropdownMenuVisual {
                    top_px: 42.0 - travel_px * (1.0 - eased),
                    left_percent: 0.0,
                    width_percent: 100.0,
                    opacity: eased,
                }
            }

            // Expands from a slightly compact menu and tightens back on close.
            Self::ScaleInOut => {
                let width_percent = 96.0 + 4.0 * eased;
                let left_percent = (100.0 - width_percent) * 0.5;

                DropdownMenuVisual {
                    top_px: 42.0,
                    left_percent,
                    width_percent,
                    opacity: eased,
                }
            }

            // Slides from the left edge of the trigger into final position.
            Self::SlideFromLeft => {
                let slide_percent = -3.0 * (1.0 - eased);

                DropdownMenuVisual {
                    top_px: 42.0,
                    left_percent: slide_percent,
                    width_percent: 100.0,
                    opacity: eased,
                }
            }

            // Slides from the right edge of the trigger into final position.
            Self::SlideFromRight => {
                let slide_percent = 3.0 * (1.0 - eased);

                DropdownMenuVisual {
                    top_px: 42.0,
                    left_percent: slide_percent,
                    width_percent: 100.0,
                    opacity: eased,
                }
            }
        }
    }
}

fn spring_step(animation: &mut DropdownMenuAnimation, dt: f32) {
    if dt <= 0.0 {
        return;
    }

    let displacement = animation.target - animation.progress;
    animation.velocity += displacement * animation.stiffness * dt;

    let damping_factor = (1.0 - animation.damping * dt).clamp(0.0, 1.0);
    animation.velocity *= damping_factor;

    animation.progress += animation.velocity * dt;

    let near_target = (animation.target - animation.progress).abs() < 0.001;
    let near_still = animation.velocity.abs() < 0.001;
    if near_target && near_still {
        animation.progress = animation.target;
        animation.velocity = 0.0;
    }
}

// ============================================================
// ACCESSIBILITY
// ============================================================

/// Keeps the trigger's expanded/label state and each option's selected
/// state in sync with `DropdownState` for screen readers.
fn dropdown_a11y_system(
    dropdown_query: Query<(Entity, &Dropdown, &DropdownState), Changed<DropdownState>>,
    mut surface_query: Query<
        (&DropdownSurface, &mut AccessibilityNode),
        Without<DropdownOptionButton>,
    >,
    mut option_query: Query<
        (&DropdownOptionButton, &mut AccessibilityNode),
        Without<DropdownSurface>,
    >,
) {
    for (entity, dropdown, state) in &dropdown_query {
        let label_text = state
            .selected
            .and_then(|index| dropdown.options.get(index))
            .map(|option| option.label.clone())
            .unwrap_or_else(|| dropdown.placeholder.clone());

        for (surface, mut node) in &mut surface_query {
            if surface.owner != entity {
                continue;
            }
            node.0.set_label(label_text.clone());
            node.0.set_expanded(state.open);
        }

        for (option, mut node) in &mut option_query {
            if option.owner != entity {
                continue;
            }
            node.0.set_selected(Some(option.index) == state.selected);
        }
    }
}

// ============================================================
// PLUGIN
// ============================================================

pub struct DropdownPlugin;

impl Plugin for DropdownPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DropdownFocus>();
        app.add_message::<DropdownEvent>().add_systems(
            Update,
            (
                dropdown_keyboard_focus_bridge_system,
                dropdown_surface_system,
                dropdown_option_system,
                dropdown_click_outside_close_system,
                dropdown_keyboard_navigation_system,
                dropdown_a11y_system,
                dropdown_visual_system,
                dropdown_menu_animation_system,
                dropdown_option_visual_system,
            )
                .chain(),
        );
    }
}
