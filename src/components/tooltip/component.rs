use bevy::prelude::*;

use crate::rendering::{GradientStop, LinearGradient, Paint, Surface};
use crate::icons::IconCommands;
use crate::components::text::{TextRole, ThemedText};

/// Adds tooltip behavior to a UI entity.
///
/// The entity must have a `Node` so Bevy can detect pointer interaction.
#[derive(Component, Debug, Clone)]
pub struct Tooltip {
    /// Text displayed inside the tooltip.
    pub text: String,

    /// How long the cursor must remain over the element
    /// before the tooltip appears.
    pub delay: f32,

    /// Where the tooltip should appear relative to the target.
    pub placement: TooltipPlacement,

    /// Whether the tooltip is currently visible.
    pub visible: bool,

    /// Internal hover timer.
    pub timer: f32,
}

impl Tooltip {
    pub fn from_config(config: TooltipConfig) -> Self {
        Self {
            text: config.text,
            delay: config.delay,
            placement: config.placement,
            visible: false,
            timer: 0.0,
        }
    }

    pub fn new(text: impl Into<String>) -> Self {
        Self::from_config(TooltipConfig::new(text))
    }

    pub fn delay(mut self, seconds: f32) -> Self {
        self.delay = seconds;
        self
    }

    pub fn placement(mut self, placement: TooltipPlacement) -> Self {
        self.placement = placement;
        self
    }
}

#[derive(Debug, Clone)]
pub struct TooltipConfig {
    pub text: String,
    pub delay: f32,
    pub placement: TooltipPlacement,
}

impl TooltipConfig {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            delay: 0.5,
            placement: TooltipPlacement::Top,
        }
    }

    pub fn delay(mut self, seconds: f32) -> Self {
        self.delay = seconds;
        self
    }

    pub fn placement(mut self, placement: TooltipPlacement) -> Self {
        self.placement = placement;
        self
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum TooltipPlacement {
    #[default]
    Top,
    Bottom,
    Left,
    Right,
}

/// Marker component for the tooltip's visual entity.
#[derive(Component)]
struct TooltipVisual;

/// Stores the relationship between a tooltip visual
/// and the UI element it belongs to.
#[derive(Component)]
struct TooltipFor(Entity);

fn estimate_tooltip_width_px(text: &str) -> f32 {
    // Approximate width for 13px text with 8px horizontal padding on both sides.
    let glyph_width = 7.0;
    let horizontal_padding = 16.0;
    (text.chars().count() as f32 * glyph_width) + horizontal_padding
}

fn tooltip_visual_node(placement: TooltipPlacement, text: &str) -> Node {
    let mut node = Node {
        position_type: PositionType::Absolute,
        border_radius: BorderRadius::all(Val::Px(6.0)),
        ..default()
    };

    let estimated_width = estimate_tooltip_width_px(text);
    let center_offset = -0.5 * estimated_width;

    match placement {
        TooltipPlacement::Top => {
            node.left = Val::Percent(50.0);
            node.bottom = Val::Percent(100.0);
            node.margin = UiRect {
                left: Val::Px(center_offset),
                bottom: Val::Px(8.0),
                ..default()
            };
        }
        TooltipPlacement::Bottom => {
            node.left = Val::Percent(50.0);
            node.top = Val::Percent(100.0);
            node.margin = UiRect {
                left: Val::Px(center_offset),
                top: Val::Px(8.0),
                ..default()
            };
        }
        TooltipPlacement::Left => {
            node.right = Val::Percent(100.0);
            node.top = Val::Px(0.0);
            node.margin = UiRect::right(Val::Px(8.0));
        }
        TooltipPlacement::Right => {
            node.left = Val::Percent(100.0);
            node.top = Val::Px(0.0);
            node.margin = UiRect::left(Val::Px(8.0));
        }
    }

    node
}

fn tooltip_arrow_icon(placement: TooltipPlacement) -> &'static str {
    match placement {
        TooltipPlacement::Top => "chevron-down",
        TooltipPlacement::Bottom => "chevron-up",
        TooltipPlacement::Left => "chevron-right",
        TooltipPlacement::Right => "chevron-left",
    }
}

fn tooltip_arrow_node(placement: TooltipPlacement) -> Node {
    let mut node = Node {
        position_type: PositionType::Absolute,
        ..default()
    };

    match placement {
        TooltipPlacement::Top => {
            node.left = Val::Percent(50.0);
            node.bottom = Val::Px(-12.0);
            node.margin = UiRect::left(Val::Px(-4.0));
        }
        TooltipPlacement::Bottom => {
            node.left = Val::Percent(50.0);
            node.top = Val::Px(-12.0);
            node.margin = UiRect::left(Val::Px(-4.0));
        }
        TooltipPlacement::Left => {
            node.right = Val::Px(-10.0);
            node.top = Val::Percent(50.0);
            node.margin = UiRect::top(Val::Px(-8.0));
        }
        TooltipPlacement::Right => {
            node.left = Val::Px(-10.0);
            node.top = Val::Percent(50.0);
            node.margin = UiRect::top(Val::Px(-8.0));
        }
    }

    node
}

pub struct TooltipPlugin;

impl Plugin for TooltipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (tooltip_hover_system, tooltip_visibility_system));
    }
}

/// Tracks hover state and tooltip delay.
fn tooltip_hover_system(time: Res<Time>, mut query: Query<(&Interaction, &mut Tooltip)>) {
    for (interaction, mut tooltip) in &mut query {
        match *interaction {
            Interaction::Hovered => {
                tooltip.timer += time.delta_secs();

                if tooltip.timer >= tooltip.delay {
                    tooltip.visible = true;
                }
            }

            Interaction::None => {
                tooltip.timer = 0.0;
                tooltip.visible = false;
            }

            Interaction::Pressed => {}
        }
    }
}

/// Creates/removes the tooltip visual based on Tooltip state.
fn tooltip_visibility_system(
    mut commands: Commands,
    tooltip_query: Query<(Entity, &Tooltip, Option<&Children>), Changed<Tooltip>>,
    visual_query: Query<(Entity, &TooltipFor)>,
) {
    for (entity, tooltip, children) in &tooltip_query {
        if tooltip.visible {
            // Don't create a second tooltip.
            let already_exists = children
                .map(|children| {
                    children.iter().any(|child| {
                        visual_query
                            .get(child)
                            .map(|(_, tooltip_for)| tooltip_for.0 == entity)
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false);

            if already_exists {
                continue;
            }

            let tooltip_text = tooltip.text.clone();
            let placement = tooltip.placement;

            commands.entity(entity).with_children(|parent| {
                parent
                    .spawn((
                        TooltipVisual,
                        TooltipFor(entity),
                        tooltip_visual_node(placement, &tooltip_text),
                    ))
                    .insert(BackgroundColor(Color::NONE))
                    .insert(Surface::rounded_rect_fill(
                        8.0,
                        Paint::linear(LinearGradient::vertical(vec![
                            GradientStop::new(0.0, Color::srgb(0.08, 0.08, 0.08)),
                            GradientStop::new(1.0, Color::srgb(0.12, 0.12, 0.14)),
                        ])),
                    ))
                    .with_children(|tooltip_parent| {
                        tooltip_parent
                            .spawn(tooltip_arrow_node(placement))
                            .with_children(|arrow| {
                                arrow.spawn_feather_sized(tooltip_arrow_icon(placement), 12.0);
                            });

                        tooltip_parent.spawn((
                            ThemedText::new(TextRole::Caption),
                            Text::new(tooltip_text),
                            TextFont {
                                font_size: FontSize::Px(13.0),
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            Node {
                                padding: UiRect::axes(Val::Px(8.0), Val::Px(5.0)),
                                ..default()
                            },
                        ));
                    });
            });
        } else {
            // Remove any tooltip visual belonging to this entity.
            for (visual_entity, tooltip_for) in &visual_query {
                if tooltip_for.0 == entity {
                    commands
                        .entity(visual_entity)
                        .despawn_related::<Children>()
                        .despawn();
                }
            }
        }
    }
}
