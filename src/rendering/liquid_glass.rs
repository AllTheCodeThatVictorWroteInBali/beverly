//! Optical layer of `Backdrop`: geometry, paint, shadows and focus stay Surface-owned.
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GlassProfile {
    Convex,
    #[default]
    Squircle,
    Concave,
    Lip,
}

/// Logical-pixel lens parameters. Tint, blur and saturation belong to `Backdrop`.
/// `press_amount` is a normalized input from the existing interaction/animation system;
/// it changes only the optical profile, never the content geometry.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LiquidGlass {
    pub enabled: bool,
    pub thickness: f32,
    pub bezel_width: f32,
    pub refractive_index: f32,
    pub specular_intensity: f32,
    pub specular_width: f32,
    pub light_direction: Vec2,
    pub fresnel: f32,
    pub chromatic_aberration: f32,
    pub profile: GlassProfile,
    pub press_amount: f32,
}

impl Default for LiquidGlass {
    fn default() -> Self {
        Self {
            enabled: true,
            thickness: 3.0,
            bezel_width: 4.5,
            refractive_index: 1.46,
            specular_intensity: 0.28,
            specular_width: 0.8,
            light_direction: Vec2::new(-0.6, -0.8),
            fresnel: 0.14,
            chromatic_aberration: 0.018,
            profile: GlassProfile::Squircle,
            press_amount: 0.0,
        }
    }
}

impl LiquidGlass {
    pub fn sanitized(self) -> Self {
        fn bounded(v: f32, fallback: f32, min: f32, max: f32) -> f32 {
            if v.is_finite() { v.clamp(min, max) } else { fallback }
        }
        let defaults = Self::default();
        Self {
            thickness: bounded(self.thickness, defaults.thickness, 0.0, 24.0),
            bezel_width: bounded(self.bezel_width, defaults.bezel_width, 0.5, 48.0),
            refractive_index: bounded(self.refractive_index, defaults.refractive_index, 1.0, 1.8),
            specular_intensity: bounded(self.specular_intensity, defaults.specular_intensity, 0.0, 0.6),
            specular_width: bounded(self.specular_width, defaults.specular_width, 0.1, 8.0),
            light_direction: self.light_direction.try_normalize().filter(|v| v.is_finite())
                .unwrap_or(defaults.light_direction),
            fresnel: bounded(self.fresnel, defaults.fresnel, 0.0, 0.4),
            chromatic_aberration: bounded(self.chromatic_aberration, defaults.chromatic_aberration, 0.0, 0.08),
            press_amount: bounded(self.press_amount, 0.0, 0.0, 1.0),
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn liquid_glass_sanitizes_every_optical_input() {
        for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -100.0, 10000.0] {
            let glass = LiquidGlass {
                thickness: value, bezel_width: value, refractive_index: value,
                specular_intensity: value, specular_width: value, fresnel: value,
                chromatic_aberration: value, press_amount: value,
                light_direction: Vec2::splat(value), ..default()
            }.sanitized();
            assert!((1.0..=1.8).contains(&glass.refractive_index));
            assert!((0.0..=24.0).contains(&glass.thickness));
            assert!((0.5..=48.0).contains(&glass.bezel_width));
            assert!((0.0..=0.6).contains(&glass.specular_intensity));
            assert!((0.1..=8.0).contains(&glass.specular_width));
            assert!((0.0..=0.4).contains(&glass.fresnel));
            assert!((0.0..=0.08).contains(&glass.chromatic_aberration));
            assert!((0.0..=1.0).contains(&glass.press_amount));
            assert!(glass.light_direction.is_normalized());
        }
    }
}