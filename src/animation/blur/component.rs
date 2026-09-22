use bevy::prelude::*;

use crate::rendering::{Backdrop, BackdropQuality, Paint, Surface};

/// Full-screen visual backdrop used behind modals, popups, sheets, and other
/// layered UI surfaces.
#[derive(Component, Clone)]
pub struct BackdropBlur {
    /// Overall opacity of the backdrop.
    pub opacity: f32,

    /// How dark the backdrop is.
    pub darkness: f32,

    /// Controls the animated softness/glow.
    pub intensity: f32,

    /// Semantic blur strength in UI units.
    pub blur: f32,

    /// Whether the backdrop is currently visible.
    pub visible: bool,
}

impl Default for BackdropBlur {
    fn default() -> Self {
        Self {
            opacity: 0.72,
            darkness: 0.55,
            intensity: 1.0,
            blur: 16.0,
            visible: true,
        }
    }
}

impl BackdropBlur {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    pub fn darkness(mut self, darkness: f32) -> Self {
        self.darkness = darkness.clamp(0.0, 1.0);
        self
    }

    pub fn intensity(mut self, intensity: f32) -> Self {
        self.intensity = intensity.max(0.0);
        self
    }

    pub fn blur(mut self, blur: f32) -> Self {
        self.blur = blur.max(0.0);
        self
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }
}

/// Plugin for the backdrop blur system.
pub struct BackdropBlurPlugin;

impl Plugin for BackdropBlurPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_backdrop_blur);
    }
}

/// Spawn a fullscreen backdrop node for a parent UI tree.
pub fn spawn_backdrop_blur(parent: &mut ChildSpawnerCommands, blur: BackdropBlur) -> Entity {
    parent
        .spawn((
            blur,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                overflow: Overflow::clip(),
                ..default()
            },
            Surface::rounded_rect_fill(0.0, Paint::solid(Color::srgba(0.02, 0.03, 0.06, 0.0))),
            ZIndex(-100),
            BackdropBlurVisual,
        ))
        .id()
}

/// Marker for the visual portion of the backdrop.
#[derive(Component)]
struct BackdropBlurVisual;

fn update_backdrop_blur(
    time: Res<Time>,
    mut query: Query<(&BackdropBlur, &mut Surface, &mut Node)>,
) {
    for (blur, mut surface, mut node) in &mut query {
        if !blur.visible {
            surface.fill = Paint::solid(Color::srgba(0.02, 0.03, 0.06, 0.0));
            surface.backdrop = None;
            continue;
        }

        let t = time.elapsed_secs();
        let pulse = ((t * 0.72).sin() * 0.03 + 1.0) * blur.intensity.clamp(0.35, 2.0);
        let alpha = (blur.opacity * pulse).clamp(0.0, 1.0);

        let red = (0.025 + blur.darkness * 0.025).clamp(0.0, 0.2);
        let green = (0.035 + blur.darkness * 0.03).clamp(0.0, 0.2);
        let blue = (0.055 + blur.darkness * 0.035).clamp(0.0, 0.2);

        surface.fill = Paint::solid(Color::srgba(red, green, blue, alpha));
        surface.backdrop = Some(
            Backdrop::new()
                .with_blur(blur.blur * blur.intensity.clamp(0.5, 2.0))
                .with_tint(Color::srgba(0.96, 0.98, 1.0, 1.0))
                .with_tint_opacity((0.08 + blur.darkness * 0.08).clamp(0.0, 0.35))
                .with_brightness((1.04 - blur.darkness * 0.25).clamp(0.7, 1.2))
                .with_saturation((1.0 - blur.darkness * 0.25).clamp(0.6, 1.1))
                .with_contrast((1.0 + blur.darkness * 0.18).clamp(0.8, 1.35))
                .with_quality(BackdropQuality::Medium),
        );

        node.left = Val::Px(0.0);
        node.right = Val::Px(0.0);
        node.top = Val::Px(0.0);
        node.bottom = Val::Px(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::BackdropBlur;

    #[test]
    fn builder_methods_clamp_and_toggle_values() {
        let blur = BackdropBlur::new()
            .opacity(1.8)
            .darkness(-0.2)
            .intensity(-0.4)
            .blur(-10.0)
            .visible(false);

        assert!((blur.opacity - 1.0).abs() < f32::EPSILON);
        assert!(blur.darkness.abs() < f32::EPSILON);
        assert!((blur.intensity - 0.0).abs() < f32::EPSILON);
        assert!((blur.blur - 0.0).abs() < f32::EPSILON);
        assert!(!blur.visible);
    }
}
