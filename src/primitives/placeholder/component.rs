#![allow(deprecated)]

use bevy::prelude::*;

use crate::primitives::placeholder::{PlaceholderEffect, PlaceholderMaterial, PlaceholderUniforms};

/// A basic layout helper for stacking placeholder elements.
#[derive(Component, Debug, Clone)]
#[deprecated(note = "Use ui::skeleton::SkeletonGroup for an accessible loading region")]
pub struct LegacySkeletonGroup {
    pub flex_direction: FlexDirection,
    pub gap: f32,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
}

#[allow(deprecated)]
impl LegacySkeletonGroup {
    pub fn new() -> Self {
        Self {
            flex_direction: FlexDirection::Column,
            gap: 12.0,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Stretch,
        }
    }

    pub fn row() -> Self {
        Self {
            flex_direction: FlexDirection::Row,
            gap: 12.0,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Center,
        }
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    pub fn flex_direction(mut self, direction: FlexDirection) -> Self {
        self.flex_direction = direction;
        self
    }
}

#[allow(deprecated)]
impl Default for LegacySkeletonGroup {
    fn default() -> Self {
        Self::new()
    }
}

/// Describes the shape/size of a placeholder.
#[derive(Debug, Clone, Copy)]
pub enum PlaceholderShape {
    /// Text-like placeholder.
    ///
    /// `font_size` is used to estimate height.
    /// `characters` is used to estimate width.
    Text { font_size: f32, characters: usize },

    /// Square placeholder.
    Square { size: f32 },

    /// Circular placeholder.
    Round { size: f32 },

    /// Arbitrary rectangle.
    Rect { width: f32, height: f32 },
}

/// Main placeholder component.
///
/// The component describes the placeholder itself.
/// Animation is handled independently by `PlaceholderEffect`.
#[derive(Component, Debug, Clone)]
#[deprecated(note = "Use ui::skeleton::Skeleton for native Surface-based loading UI")]
pub struct Placeholder {
    pub shape: PlaceholderShape,

    /// Animation/highlight effect.
    pub effect: PlaceholderEffect,

    /// Base skeleton color.
    pub color: Color,

    /// Highlight color used by effects.
    pub highlight_color: Color,

    /// Animation speed.
    pub speed: f32,

    /// Approximate width of a character as a fraction
    /// of the font size.
    pub character_width: f32,

    /// Whether the placeholder is currently active.
    pub enabled: bool,
}

impl Default for Placeholder {
    fn default() -> Self {
        Self {
            shape: PlaceholderShape::Rect {
                width: 100.0,
                height: 16.0,
            },

            effect: PlaceholderEffect::Shimmer,

            color: Color::srgb(0.88, 0.88, 0.89),

            highlight_color: Color::srgba(1.0, 1.0, 1.0, 0.45),

            speed: 1.5,

            character_width: 0.52,

            enabled: true,
        }
    }
}

impl Placeholder {
    // ---------------------------------------------------------
    // Constructors
    // ---------------------------------------------------------

    pub fn text(font_size: f32, characters: usize) -> Self {
        Self {
            shape: PlaceholderShape::Text {
                font_size,
                characters,
            },
            ..default()
        }
    }

    pub fn square(size: f32) -> Self {
        Self {
            shape: PlaceholderShape::Square { size },
            ..default()
        }
    }

    pub fn round(size: f32) -> Self {
        Self {
            shape: PlaceholderShape::Round { size },
            ..default()
        }
    }

    pub fn rect(width: f32, height: f32) -> Self {
        Self {
            shape: PlaceholderShape::Rect { width, height },
            ..default()
        }
    }

    // ---------------------------------------------------------
    // Effects
    // ---------------------------------------------------------

    pub fn effect(mut self, effect: PlaceholderEffect) -> Self {
        self.effect = effect;
        self
    }

    pub fn shimmer(mut self) -> Self {
        self.effect = PlaceholderEffect::Shimmer;
        self
    }

    pub fn pulse(mut self) -> Self {
        self.effect = PlaceholderEffect::Pulse;
        self
    }

    pub fn fade(mut self) -> Self {
        self.effect = PlaceholderEffect::Fade;
        self
    }

    pub fn light_reveal(mut self) -> Self {
        self.effect = PlaceholderEffect::LightReveal;
        self
    }

    pub fn circular_reveal(mut self) -> Self {
        self.effect = PlaceholderEffect::CircularReveal;
        self
    }

    pub fn static_placeholder(mut self) -> Self {
        self.effect = PlaceholderEffect::None;
        self
    }

    // ---------------------------------------------------------
    // Styling
    // ---------------------------------------------------------

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn highlight_color(mut self, color: Color) -> Self {
        self.highlight_color = color;
        self
    }

    pub fn speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    pub fn character_width(mut self, width: f32) -> Self {
        self.character_width = width;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Computes the layout dimensions for the placeholder `Node` based on `PlaceholderShape`.
    pub fn to_node_components(&self) -> Node {
        let mut node = Node::default();

        match self.shape {
            PlaceholderShape::Text {
                font_size,
                characters,
            } => {
                let width = font_size * self.character_width * characters as f32;
                node.width = Val::Px(width);
                node.height = Val::Px(font_size * 1.25);
                node.border_radius = BorderRadius::all(Val::Px(3.0));
            }
            PlaceholderShape::Square { size } => {
                node.width = Val::Px(size);
                node.height = Val::Px(size);
                node.border_radius = BorderRadius::all(Val::Px(6.0));
            }
            PlaceholderShape::Round { size } => {
                node.width = Val::Px(size);
                node.height = Val::Px(size);
                node.border_radius = BorderRadius::all(Val::Percent(50.0));
            }
            PlaceholderShape::Rect { width, height } => {
                node.width = Val::Px(width);
                node.height = Val::Px(height);
                node.border_radius = BorderRadius::all(Val::Px(6.0));
            }
        }

        node
    }
}

/// Automatically attach shader-backed material to placeholders once they are added.
pub(crate) fn setup_placeholders(
    mut commands: Commands,
    mut materials: ResMut<Assets<PlaceholderMaterial>>,
    query: Query<
        (Entity, &Placeholder),
        (
            Added<Placeholder>,
            Without<MaterialNode<PlaceholderMaterial>>,
        ),
    >,
) {
    for (entity, placeholder) in &query {
        let node = placeholder.to_node_components();
        let material = materials.add(PlaceholderMaterial {
            uniforms: PlaceholderUniforms {
                base_color: placeholder.color.to_linear(),
                highlight_color: placeholder.highlight_color.to_linear(),
                time: 0.0,
                effect: placeholder.effect.shader_value(),
                intensity: 1.0,
                speed: placeholder.speed,
                angle: 0.0,
                width: 0.25,
            },
        });

        commands.entity(entity).insert((
            node,
            MaterialNode(material),
            PlaceholderRoot,
            PlaceholderAnimation::default(),
        ));
    }
}

/// Internal animation state.
#[derive(Component, Debug)]
pub struct PlaceholderAnimation {
    pub time: f32,
    pub progress: f32,
}

impl Default for PlaceholderAnimation {
    fn default() -> Self {
        Self {
            time: 0.0,
            progress: 0.0,
        }
    }
}

/// Marks the root entity of a placeholder.
#[derive(Component)]
pub struct PlaceholderRoot;
