use bevy::prelude::*;

use crate::components::text::{TextRole, ThemedText};
use crate::rendering::{Paint, Surface};
use crate::theme::{ThemeMode, ThemeResource};

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

#[derive(Component)]
struct TooltipArrow;

#[derive(Component)]
struct TooltipBubble;

#[derive(Component)]
struct TooltipLabel;

/// Stores the relationship between a tooltip visual
/// and the UI element it belongs to.
#[derive(Component)]
struct TooltipFor(Entity);

const TOOLTIP_MAX_WIDTH: f32 = 260.0;

/// Fixed-width strip that places the bubble; it never constrains the bubble's own width.
fn tooltip_visual_node(placement: TooltipPlacement) -> Node {
    let mut node = Node {
        position_type: PositionType::Absolute,
        width: Val::Px(TOOLTIP_MAX_WIDTH),
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        ..default()
    };

    match placement {
        TooltipPlacement::Top => {
            node.left = Val::Percent(50.0);
            node.bottom = Val::Percent(100.0);
            node.justify_content = JustifyContent::Center;
            node.margin = UiRect {
                left: Val::Px(-TOOLTIP_MAX_WIDTH * 0.5),
                bottom: Val::Px(8.0),
                ..default()
            };
        }
        TooltipPlacement::Bottom => {
            node.left = Val::Percent(50.0);
            node.top = Val::Percent(100.0);
            node.justify_content = JustifyContent::Center;
            node.margin = UiRect {
                left: Val::Px(-TOOLTIP_MAX_WIDTH * 0.5),
                top: Val::Px(8.0),
                ..default()
            };
        }
        TooltipPlacement::Left => {
            node.right = Val::Percent(100.0);
            node.top = Val::Percent(50.0);
            node.justify_content = JustifyContent::FlexEnd;
            node.margin = UiRect::right(Val::Px(8.0));
        }
        TooltipPlacement::Right => {
            node.left = Val::Percent(100.0);
            node.top = Val::Percent(50.0);
            node.justify_content = JustifyContent::FlexStart;
            node.margin = UiRect::left(Val::Px(8.0));
        }
    }

    node
}

fn tooltip_bubble_node() -> Node {
    Node {
        flex_shrink: 0.0,
        max_width: Val::Px(TOOLTIP_MAX_WIDTH),
        padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
        border_radius: BorderRadius::all(Val::Px(6.0)),
        ..default()
    }
}

fn tooltip_translation(placement: TooltipPlacement) -> Val2 {
    match placement {
        TooltipPlacement::Top | TooltipPlacement::Bottom => Val2::ZERO,
        TooltipPlacement::Left | TooltipPlacement::Right => Val2::percent(0.0, -50.0),
    }
}

fn tooltip_arrow_node(placement: TooltipPlacement) -> Node {
    let mut node = Node {
        position_type: PositionType::Absolute,
        width: Val::Px(8.0),
        height: Val::Px(8.0),
        ..default()
    };

    match placement {
        TooltipPlacement::Top => {
            node.left = Val::Percent(50.0);
            node.bottom = Val::Px(-4.0);
            node.margin = UiRect::left(Val::Px(-4.0));
        }
        TooltipPlacement::Bottom => {
            node.left = Val::Percent(50.0);
            node.top = Val::Px(-4.0);
            node.margin = UiRect::left(Val::Px(-4.0));
        }
        TooltipPlacement::Left => {
            node.right = Val::Px(-4.0);
            node.top = Val::Percent(50.0);
            node.margin = UiRect::top(Val::Px(-4.0));
        }
        TooltipPlacement::Right => {
            node.left = Val::Px(-4.0);
            node.top = Val::Percent(50.0);
            node.margin = UiRect::top(Val::Px(-4.0));
        }
    }

    node
}

fn tooltip_colors(mode: ThemeMode) -> (Color, Color) {
    match mode {
        ThemeMode::Light => (Color::srgb_u8(24, 24, 27), Color::srgb_u8(250, 250, 250)),
        ThemeMode::Dark => (Color::srgb_u8(250, 250, 250), Color::srgb_u8(24, 24, 27)),
    }
}

fn tooltip_theme_system(
    theme: Res<ThemeResource>,
    mut surfaces: Query<&mut Surface, Or<(With<TooltipBubble>, With<TooltipArrow>)>>,
    mut labels: Query<(&mut ThemedText, &mut TextColor), With<TooltipLabel>>,
) {
    let (background, foreground) = tooltip_colors(theme.current.mode);
    for mut surface in &mut surfaces {
        if surface.fill != Paint::solid(background) {
            surface.fill = Paint::solid(background);
        }
    }
    for (mut themed, mut color) in &mut labels {
        if themed.color_override != Some(foreground) {
            themed.color_override = Some(foreground);
        }
        if color.0 != foreground {
            color.0 = foreground;
        }
    }
}

#[cfg(test)]
mod style_tests {
    use super::*;

    #[test]
    fn bubble_sizes_to_its_text_and_is_not_limited_by_the_target_width() {
        for placement in [
            TooltipPlacement::Top,
            TooltipPlacement::Bottom,
            TooltipPlacement::Left,
            TooltipPlacement::Right,
        ] {
            let anchor = tooltip_visual_node(placement);
            assert_eq!(anchor.width, Val::Px(TOOLTIP_MAX_WIDTH));
            let bubble = tooltip_bubble_node();
            assert_eq!(bubble.flex_shrink, 0.0);
            assert_eq!(bubble.width, Val::Auto);
            assert_eq!(bubble.max_width, Val::Px(TOOLTIP_MAX_WIDTH));
        }
        let top = tooltip_visual_node(TooltipPlacement::Top);
        assert_eq!(top.justify_content, JustifyContent::Center);
        assert_eq!(top.margin.left, Val::Px(-TOOLTIP_MAX_WIDTH * 0.5));
    }

    #[test]
    fn tooltip_bubble_pointer_and_text_follow_the_inverse_theme_palette() {
        let mut app = App::new();
        app.insert_resource(ThemeResource {
            current: crate::theme::light_theme(),
        })
        .add_systems(
            Update,
            (tooltip_visibility_system, tooltip_theme_system).chain(),
        );
        let mut tooltip = Tooltip::new("One tin. No sharing clause.");
        tooltip.visible = true;
        app.world_mut().spawn((Node::default(), tooltip));
        for theme in [crate::theme::light_theme(), crate::theme::dark_theme()] {
            let (background, foreground) = tooltip_colors(theme.mode);
            app.world_mut().resource_mut::<ThemeResource>().current = theme;
            app.update();
            let mut bubbles = app
                .world_mut()
                .query_filtered::<(&Node, &Surface), With<TooltipBubble>>();
            let (node, surface) = bubbles.single(app.world()).unwrap();
            assert_eq!(surface.fill, Paint::solid(background));
            assert_eq!(node.padding, UiRect::axes(Val::Px(12.0), Val::Px(6.0)));
            let mut arrows = app
                .world_mut()
                .query_filtered::<&Surface, With<TooltipArrow>>();
            assert_eq!(
                arrows.single(app.world()).unwrap().fill,
                Paint::solid(background)
            );
            let mut labels = app
                .world_mut()
                .query_filtered::<(&ThemedText, &TextColor), With<TooltipLabel>>();
            let (themed, color) = labels.single(app.world()).unwrap();
            assert_eq!(themed.size_override, Some(12.0));
            assert_eq!(themed.color_override, Some(foreground));
            assert_eq!(color.0, foreground);
        }
    }
}

pub struct TooltipPlugin;

impl Plugin for TooltipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                tooltip_hover_system,
                tooltip_visibility_system,
                tooltip_theme_system,
            )
                .chain(),
        );
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
    theme: Res<ThemeResource>,
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
            let (background, foreground) = tooltip_colors(theme.current.mode);

            commands.entity(entity).with_children(|parent| {
                parent
                    .spawn((
                        TooltipVisual,
                        TooltipFor(entity),
                        tooltip_visual_node(placement),
                        UiTransform {
                            translation: tooltip_translation(placement),
                            ..default()
                        },
                        ZIndex(2000),
                        Pickable::IGNORE,
                    ))
                    .with_children(|tooltip_parent| {
                        tooltip_parent
                            .spawn((
                                TooltipBubble,
                                tooltip_bubble_node(),
                                BackgroundColor(Color::NONE),
                                Surface::rounded_rect_fill(6.0, Paint::solid(background)),
                                Pickable::IGNORE,
                            ))
                            .with_children(|bubble| {
                                bubble.spawn((
                                    TooltipArrow,
                                    tooltip_arrow_node(placement),
                                    UiTransform {
                                        rotation: Rot2::degrees(45.0),
                                        ..default()
                                    },
                                    Surface::rounded_rect_fill(1.0, Paint::solid(background)),
                                    Pickable::IGNORE,
                                ));

                                bubble.spawn((
                                    TooltipLabel,
                                    ThemedText::new(TextRole::Caption)
                                        .size(12.0)
                                        .color(foreground),
                                    Text::new(tooltip_text),
                                    TextFont {
                                        font_size: FontSize::Px(12.0),
                                        ..default()
                                    },
                                    TextColor(foreground),
                                    TextLayout::linebreak(LineBreak::WordOrCharacter),
                                    Pickable::IGNORE,
                                ));
                            });
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
