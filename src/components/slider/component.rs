use bevy::a11y::AccessibilityNode;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use bevy::window::{CursorIcon, PrimaryWindow, SystemCursorIcon};

use crate::rendering::{OuterShadow, Paint, Surface};
use crate::primitives::a11y;
use crate::components::text::{TextRole, ThemedText};
use crate::theme::ThemeResource;

// ─────────────────────────────────────────────────────────────────────────────
// Slider
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Component, Debug, Clone)]
pub struct Slider {
    pub min: f32,
    pub max: f32,
    pub value: f32,
    pub step: Option<f32>,
    pub disabled: bool,
    pub accessible_label: Option<String>,
}

impl Slider {
    pub fn new(min: f32, max: f32) -> Self {
        Self {
            min,
            max,
            value: min,
            step: None,
            disabled: false,
            accessible_label: None,
        }
    }

    pub fn value(mut self, value: f32) -> Self {
        self.value = value.clamp(self.min, self.max);
        self
    }

    pub fn step(mut self, step: f32) -> Self {
        self.step = Some(step);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets the accessible name announced by screen readers (defaults to "Slider").
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.accessible_label = Some(label.into());
        self
    }

    pub fn normalized(&self) -> f32 {
        if self.max <= self.min {
            return 0.0;
        }

        ((self.value - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Visual Components
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct SliderTrack;

#[derive(Component)]
pub struct SliderFill;

#[derive(Component)]
pub struct SliderThumb;

#[derive(Component)]
pub struct SliderValue;

#[derive(Component, Default)]
pub struct SliderInteraction {
    pub hovered: bool,
    pub dragging: bool,
}

#[derive(Component)]
pub struct SliderVisual {
    pub displayed_value: f32,
    pub thumb_base_size: f32,
}

#[derive(Component)]
struct SliderParts {
    track_slot: Entity,
    fill: Entity,
    thumb: Entity,
    value_text: Option<Entity>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Event
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Event, Message, Debug, Clone, Copy)]
pub struct SliderChanged {
    pub entity: Entity,
    pub value: f32,
    pub dragging: bool,
}

// ─────────────────────────────────────────────────────────────────────────────
// Style
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct SliderStyle {
    pub width: f32,
    pub track_height: f32,
    pub thumb_size: f32,

    pub track_color: Color,
    pub fill_color: Color,
    pub thumb_color: Color,

    pub show_value: bool,
}

impl Default for SliderStyle {
    fn default() -> Self {
        Self {
            width: 260.0,
            track_height: 6.0,
            thumb_size: 20.0,

            track_color: Color::srgba(1.0, 1.0, 1.0, 0.12),
            fill_color: Color::srgba(1.0, 1.0, 1.0, 0.85),
            thumb_color: Color::srgba(1.0, 1.0, 1.0, 0.95),

            show_value: false,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Spawn
// ─────────────────────────────────────────────────────────────────────────────

pub fn spawn_slider(
    parent: &mut ChildSpawnerCommands,
    slider: Slider,
    style: SliderStyle,
    theme: &ThemeResource,
) -> Entity {
    let colors = theme.current.colors;
    let normalized = slider.normalized();
    let percent = normalized * 100.0;
    let mut track_slot_entity = None;
    let mut fill_entity = None;
    let mut thumb_entity = None;
    let mut value_text = None;

    let mut slider_entity = parent.spawn((
        Button,
        a11y::TabIndex(if slider.disabled { -1 } else { 0 }),
        a11y::slider_node(
            slider
                .accessible_label
                .clone()
                .unwrap_or_else(|| "Slider".to_string()),
            slider.value,
            slider.min,
            slider.max,
        ),
        Node {
            width: Val::Px(style.width),
            min_height: Val::Px(36.0),
            position_type: PositionType::Relative,
            align_items: AlignItems::Center,
            column_gap: px(12.0),
            ..default()
        },
        slider.clone(),
        SliderInteraction::default(),
        SliderVisual {
            displayed_value: slider.value,
            thumb_base_size: style.thumb_size,
        },
    ));

    let slider_entity_id = slider_entity.id();

    slider_entity.with_children(|root| {
        let mut track_slot_node = root.spawn((
            Node {
                flex_grow: 1.0,
                height: Val::Px(32.0),
                position_type: PositionType::Relative,
                ..default()
            },
            RelativeCursorPosition::default(),
            BackgroundColor(Color::NONE),
        ));

        track_slot_entity = Some(track_slot_node.id());

        track_slot_node.with_children(|track_slot| {
            // Track
            let track_surface = Surface::rounded_rect_fill(style.track_height * 0.5, Paint::solid(style.track_color))
                .with_mask(crate::rendering::Mask::rounded_rect(style.track_height * 0.5).with_opacity(1.0))
                .uniform_border(1.0, Paint::solid(colors.border.with_alpha(0.55)));
            track_slot
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(style.track_height),
                        position_type: PositionType::Absolute,
                        top: Val::Px((32.0 - style.track_height) * 0.5),
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BackgroundColor(style.track_color.with_alpha(0.9)),
                    track_surface,
                    SliderTrack,
                ))
                .with_children(|track| {
                    // Fill
                    fill_entity = Some(
                        track
                            .spawn((
                                Node {
                                    width: Val::Percent(percent),
                                    height: Val::Percent(100.0),
                                    border_radius: BorderRadius::MAX,
                                    ..default()
                                },
                                BackgroundColor(style.fill_color.with_alpha(0.96)),
                                Surface::rounded_rect_fill(style.track_height * 0.5, Paint::solid(style.fill_color))
                                    .with_mask(crate::rendering::Mask::rounded_rect(style.track_height * 0.5).with_opacity(1.0))
                                    .uniform_border(1.0, Paint::solid(colors.border.with_alpha(0.45))),
                                SliderFill,
                            ))
                            .id(),
                    );
                });

            // Thumb
            let thumb_builder = track_slot.spawn((
                Node {
                    width: Val::Px(style.thumb_size),
                    height: Val::Px(style.thumb_size),
                    position_type: PositionType::Absolute,
                    left: Val::Percent(percent),
                    top: Val::Px((32.0 - style.thumb_size) * 0.5),
                    border: UiRect::all(px(2.0)),
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BackgroundColor(style.thumb_color.with_alpha(0.96)),
                BorderColor::all(colors.border.with_alpha(0.45)),
                Surface::rounded_rect_fill(style.thumb_size * 0.5, Paint::solid(style.thumb_color))
                    .with_mask(crate::rendering::Mask::rounded_rect(style.thumb_size * 0.5).with_opacity(1.0))
                    .outer_shadow(OuterShadow::small(Color::BLACK).with_opacity(0.75))
                    .uniform_border(2.0, Paint::solid(colors.primary.with_alpha(0.7))),
                RelativeCursorPosition::default(),
                ZIndex(2),
                SliderThumb,
            ));

            thumb_entity = Some(thumb_builder.id());
        });

        if style.show_value {
            value_text = Some(
                root.spawn((
                    SliderValue,
                    ThemedText::new(TextRole::Label),
                    Text::new(format_slider_value(slider.value, slider.step)),
                    TextFont {
                        font_size: FontSize::Px(16.0),
                        ..default()
                    },
                    TextColor(colors.text.with_alpha(0.92)),
                    Node {
                        min_width: Val::Px(56.0),
                        ..default()
                    },
                ))
                .id(),
            );
        }
    });

    let track_slot = track_slot_entity.expect("slider track slot should be created");
    let fill = fill_entity.expect("slider fill should be created");
    let thumb = thumb_entity.expect("slider thumb should be created");

    slider_entity.insert(SliderParts {
        track_slot,
        fill,
        thumb,
        value_text,
    });

    slider_entity_id
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn apply_step(value: f32, step: Option<f32>) -> f32 {
    match step {
        Some(step) if step > 0.0 => (value / step).round() * step,
        _ => value,
    }
}

fn value_to_percent(value: f32, min: f32, max: f32) -> f32 {
    if max <= min {
        return 0.0;
    }

    ((value - min) / (max - min) * 100.0).clamp(0.0, 100.0)
}

fn approach(current: f32, target: f32, speed: f32, dt: f32) -> f32 {
    current + (target - current) * (1.0 - (-speed * dt).exp())
}

fn format_slider_value(value: f32, step: Option<f32>) -> String {
    match step {
        Some(step) if step >= 1.0 => format!("{value:.0}"),
        Some(step) if step >= 0.1 => format!("{value:.1}"),
        _ => format!("{value:.2}"),
    }
}

fn over_thumb(relative: &RelativeCursorPosition, thumb_size: Vec2) -> bool {
    let Some(normalized) = relative.normalized else {
        return false;
    };

    let padding = 10.0;
    let hit_half_x = 0.5 + padding / thumb_size.x.max(1.0);
    let hit_half_y = 0.5 + padding / thumb_size.y.max(1.0);

    normalized.x.abs() <= hit_half_x && normalized.y.abs() <= hit_half_y
}

fn slider_value_from_track_normalized(slider: &Slider, normalized_x: f32) -> f32 {
    if slider.max <= slider.min {
        return slider.value;
    }

    let normalized = (normalized_x + 0.5).clamp(0.0, 1.0);
    let raw = slider.min + normalized * (slider.max - slider.min);
    apply_step(raw, slider.step).clamp(slider.min, slider.max)
}

// ─────────────────────────────────────────────────────────────────────────────
// Update Visuals
// ─────────────────────────────────────────────────────────────────────────────

fn update_slider_visuals(
    time: Res<Time>,
    theme: Res<ThemeResource>,
    mut sliders: Query<(&Slider, &mut SliderVisual, &SliderParts, &SliderInteraction)>,
    mut surface_queries: ParamSet<(
        Query<(&mut Node, &mut Surface), With<SliderFill>>,
        Query<(&mut Node, &mut Surface), (With<SliderThumb>, Without<SliderFill>)>,
    )>,
    mut values: Query<&mut Text, With<SliderValue>>,
) {
    let colors = theme.current.colors;

    for (slider, mut visual, parts, interaction) in sliders.iter_mut() {
        visual.displayed_value = slider.value;

        let percent = value_to_percent(visual.displayed_value, slider.min, slider.max);

        if let Ok((mut node, mut surface)) = surface_queries.p0().get_mut(parts.fill) {
            node.width = Val::Percent(percent);
            surface.fill = Paint::solid(colors.primary.with_alpha(0.95));
        }

        if let Ok((mut thumb_node, mut surface)) = surface_queries.p1().get_mut(parts.thumb) {
            let base = visual.thumb_base_size;
            let target_size = if interaction.dragging {
                base + 6.0
            } else if interaction.hovered {
                base + 3.0
            } else {
                base
            };

            let current_size = match thumb_node.width {
                Val::Px(value) => value,
                _ => base,
            };

            let size = approach(current_size, target_size, 16.0, time.delta_secs());
            thumb_node.width = Val::Px(size);
            thumb_node.height = Val::Px(size);
            thumb_node.left = Val::Percent(percent);

            surface.fill = Paint::solid(colors.surface_elevated);
            let shadow_opacity = if interaction.dragging {
                0.34
            } else if interaction.hovered {
                0.28
            } else {
                0.20
            };
            surface.effects.outer_shadow = Some(OuterShadow::small(Color::BLACK).with_opacity(shadow_opacity));
            if let Some(border) = surface.border.as_mut() {
                border.paint = Paint::solid(if interaction.dragging {
                    colors.focus.with_alpha(0.60)
                } else if interaction.hovered {
                    colors.focus.with_alpha(0.35)
                } else {
                    colors.border.with_alpha(0.45)
                });
            }
        }

        if let Some(value_entity) = parts.value_text {
            if let Ok(mut text) = values.get_mut(value_entity) {
                *text = Text::new(format_slider_value(slider.value, slider.step));
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Interaction
// ─────────────────────────────────────────────────────────────────────────────

fn slider_interaction(
    mut sliders: Query<(Entity, &mut Slider, &mut SliderInteraction, &SliderParts)>,
    cursors: Query<(&RelativeCursorPosition, &ComputedNode)>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut changed: MessageWriter<SliderChanged>,
) {
    let just_pressed = mouse.just_pressed(MouseButton::Left);
    let just_released = mouse.just_released(MouseButton::Left);
    let is_pressed = mouse.pressed(MouseButton::Left);

    for (entity, mut slider, mut interaction, parts) in sliders.iter_mut() {
        let _was_dragging = interaction.dragging;

        if just_released || !is_pressed {
            interaction.dragging = false;
        }

        if slider.disabled {
            interaction.hovered = false;
            continue;
        }

        let Ok((track_cursor, _)) = cursors.get(parts.track_slot) else {
            interaction.hovered = false;
            continue;
        };
        let Ok((thumb_cursor, thumb_node)) = cursors.get(parts.thumb) else {
            interaction.hovered = false;
            continue;
        };

        let over_track = track_cursor.cursor_over();
        let over_thumb = over_thumb(thumb_cursor, thumb_node.size());

        interaction.hovered = over_thumb || interaction.dragging;

        if just_pressed && over_thumb {
            interaction.dragging = true;
        } else if just_pressed && over_track {
            let Some(normalized) = track_cursor.normalized else {
                continue;
            };

            let value = slider_value_from_track_normalized(&slider, normalized.x);

            if (value - slider.value).abs() > f32::EPSILON {
                slider.value = value;
                changed.write(SliderChanged {
                    entity,
                    value,
                    dragging: false,
                });
            }
        }

        if interaction.dragging && is_pressed {
            let Some(normalized) = track_cursor.normalized else {
                continue;
            };

            let value = slider_value_from_track_normalized(&slider, normalized.x);

            if (value - slider.value).abs() > f32::EPSILON {
                slider.value = value;
                changed.write(SliderChanged {
                    entity,
                    value,
                    dragging: true,
                });
            }
        }

        if just_released {
            let committed_value = slider.value;
            changed.write(SliderChanged {
                entity,
                value: committed_value,
                dragging: false,
            });
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Hover / Drag Animation
// ─────────────────────────────────────────────────────────────────────────────

fn slider_cursor_icon(
    sliders: Query<(&Slider, &SliderInteraction), With<Slider>>,
    mut windows: Query<&mut CursorIcon, With<PrimaryWindow>>,
) {
    let Ok(mut icon) = windows.single_mut() else {
        return;
    };

    let mut hovering_handle = false;
    let mut dragging_handle = false;

    for (slider, interaction) in sliders.iter() {
        if slider.disabled {
            continue;
        }

        if interaction.dragging {
            dragging_handle = true;
            break;
        }

        if interaction.hovered {
            hovering_handle = true;
        }
    }

    let target = if dragging_handle {
        SystemCursorIcon::Grabbing
    } else if hovering_handle {
        SystemCursorIcon::Grab
    } else {
        SystemCursorIcon::Default
    };

    let already_set = matches!(&*icon, CursorIcon::System(current) if *current == target);
    if !already_set {
        *icon = target.into();
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Accessibility
// ─────────────────────────────────────────────────────────────────────────────

/// Keeps the AccessKit numeric value / disabled state in sync for screen readers.
fn slider_a11y_system(mut sliders: Query<(&Slider, &mut AccessibilityNode), Changed<Slider>>) {
    for (slider, mut node) in &mut sliders {
        node.0.set_numeric_value(slider.value as f64);
        a11y::set_disabled(&mut node, slider.disabled);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Plugin
// ─────────────────────────────────────────────────────────────────────────────

pub struct SliderPlugin;

impl Plugin for SliderPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SliderChanged>()
            .add_systems(
                Update,
                (
                    slider_interaction,
                    slider_cursor_icon,
                    slider_a11y_system,
                ),
            )
            .add_systems(PostUpdate, update_slider_visuals);
    }
}
