#![allow(deprecated)] // Legacy plugin is opt-in; native loading uses SkeletonPlugin.
use bevy::prelude::*;

use crate::rendering::{Paint, Surface};
use super::{
    Placeholder, PlaceholderAnimation, PlaceholderMaterial, PlaceholderRoot, PlaceholderShape,
    PlaceholderUniforms, calculate_effect,
};

/// Spawn a placeholder and return its root entity.
pub fn spawn_placeholder(
    commands: &mut Commands,
    materials: &mut Assets<PlaceholderMaterial>,
    placeholder: Placeholder,
) -> Entity {
    let node = placeholder.to_node_components();
    let material = materials.add(PlaceholderMaterial {
        uniforms: PlaceholderUniforms {
            base_color: placeholder.color.to_linear(),
            highlight_color: placeholder.highlight_color.to_linear(),
            time: 0.0,
            effect: placeholder.effect.shader_value(),
            intensity: 0.85,
            speed: placeholder.speed,
            angle: 0.0,
            width: 0.22,
        },
    });

    commands
        .spawn((
            placeholder,
            PlaceholderRoot,
            PlaceholderAnimation::default(),
            node,
            MaterialNode(material),
        ))
        .id()
}

/// Calculates dimensions from semantic shape information.
#[allow(dead_code)]
fn dimensions(placeholder: &Placeholder) -> (f32, f32, f32) {
    match placeholder.shape {
        PlaceholderShape::Text {
            font_size,
            characters,
        } => {
            let width = font_size * placeholder.character_width * characters as f32;

            let height = font_size * 1.25;

            (width, height, height * 0.25)
        }

        PlaceholderShape::Square { size } => (size, size, size * 0.12),

        PlaceholderShape::Round { size } => (size, size, size / 2.0),

        PlaceholderShape::Rect { width, height } => (width, height, height * 0.20),
    }
}

/// Advances placeholder animations.
pub fn animate_placeholders(
    time: Res<Time>,
    mut query: Query<(&Placeholder, &mut PlaceholderAnimation)>,
) {
    for (placeholder, mut animation) in &mut query {
        if !placeholder.enabled {
            continue;
        }

        animation.time += time.delta_secs() * placeholder.speed;

        animation.progress = animation.time.fract();
    }
}

/// Updates the underlying shader material used by the placeholder.
pub fn update_placeholder_materials(
    time: Res<Time>,
    mut materials: ResMut<Assets<PlaceholderMaterial>>,
    query: Query<(&Placeholder, &MaterialNode<PlaceholderMaterial>)>,
) {
    for (placeholder, material_node) in &query {
        let Some(mut material) = materials.get_mut(&material_node.0) else {
            continue;
        };

        material.uniforms.time = time.elapsed_secs();
        material.uniforms.effect = placeholder.effect.shader_value();
        material.uniforms.speed = placeholder.speed;
        material.uniforms.base_color = placeholder.color.to_linear();
        material.uniforms.highlight_color = placeholder.highlight_color.to_linear();
        material.uniforms.intensity = 0.85;
        material.uniforms.angle = 0.0;
        material.uniforms.width = 0.22;
    }
}

/// Applies the calculated effect to the placeholder.
///
/// This is deliberately kept separate from animation state so
/// the renderer can eventually be replaced with a shader.
pub fn update_placeholder_visuals(
    mut query: Query<(&Placeholder, &PlaceholderAnimation, &mut Surface)>,
) {
    for (placeholder, animation, mut surface) in &mut query {
        if !placeholder.enabled {
            surface.fill = Paint::solid(Color::NONE);
            continue;
        }

        let visual = calculate_effect(placeholder.effect, animation.time);

        let base = color_to_linear(placeholder.color);

        let brightness = visual.brightness;

        surface.fill = Paint::solid(Color::linear_rgba(
            base[0] * brightness,
            base[1] * brightness,
            base[2] * brightness,
            base[3],
        ));
    }
}

/// Converts Color into a simple linear representation.
///
/// This helper keeps color manipulation localized.
fn color_to_linear(color: Color) -> [f32; 4] {
    color.to_linear().to_f32_array()
}
