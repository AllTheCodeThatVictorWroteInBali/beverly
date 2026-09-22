use bevy::prelude::*;
use super::liquid_glass::LiquidGlass;

const MAX_BACKDROP_BLUR: f32 = 128.0;
const MIN_COLOR_FACTOR: f32 = 0.0;
const MAX_COLOR_FACTOR: f32 = 2.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BackdropQuality {
    Low,
    #[default]
    Medium,
    High,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BackdropDebugView {
    #[default]
    Final,
    Source,
    Blurred,
    Mask,
}

/// Backdrop describes effects that operate on already-rendered pixels behind
/// a surface rather than on the surface's own fill.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Backdrop {
    pub blur: f32,
    pub tint: Color,
    pub tint_opacity: f32,
    pub brightness: f32,
    pub saturation: f32,
    pub contrast: f32,
    pub quality: BackdropQuality,
    pub liquid_glass: Option<LiquidGlass>,
}

impl Default for Backdrop {
    fn default() -> Self {
        Self {
            blur: 0.0,
            tint: Color::WHITE,
            tint_opacity: 0.0,
            brightness: 1.0,
            saturation: 1.0,
            contrast: 1.0,
            quality: BackdropQuality::Medium,
            liquid_glass: None,
        }
    }
}

impl Backdrop {
    pub fn with_liquid_glass(mut self, glass: LiquidGlass) -> Self {
        self.liquid_glass = Some(glass);
        self
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_blur(mut self, blur: f32) -> Self {
        self.blur = blur;
        self
    }

    pub fn with_tint(mut self, tint: Color) -> Self {
        self.tint = tint;
        self
    }

    pub fn with_tint_opacity(mut self, opacity: f32) -> Self {
        self.tint_opacity = opacity;
        self
    }

    pub fn with_brightness(mut self, brightness: f32) -> Self {
        self.brightness = brightness;
        self
    }

    pub fn with_saturation(mut self, saturation: f32) -> Self {
        self.saturation = saturation;
        self
    }

    pub fn with_contrast(mut self, contrast: f32) -> Self {
        self.contrast = contrast;
        self
    }

    pub fn with_quality(mut self, quality: BackdropQuality) -> Self {
        self.quality = quality;
        self
    }

    pub fn sanitized(self) -> Self {
        Self {
            blur: sanitize_non_negative(self.blur, 0.0).min(MAX_BACKDROP_BLUR),
            tint: self.tint,
            tint_opacity: sanitize_unit(self.tint_opacity, 0.0),
            brightness: sanitize_factor(self.brightness, 1.0),
            saturation: sanitize_factor(self.saturation, 1.0),
            contrast: sanitize_factor(self.contrast, 1.0),
            quality: self.quality,
            liquid_glass: self.liquid_glass.map(LiquidGlass::sanitized),
        }
    }

    pub fn is_active(self) -> bool {
        let value = self.sanitized();
        value.blur > 0.0
            || value.liquid_glass.is_some_and(|glass| glass.enabled)
            || value.tint_opacity > 0.0
            || (value.brightness - 1.0).abs() > f32::EPSILON
            || (value.saturation - 1.0).abs() > f32::EPSILON
            || (value.contrast - 1.0).abs() > f32::EPSILON
    }
}

fn sanitize_non_negative(value: f32, fallback: f32) -> f32 {
    if value.is_nan() {
        fallback.max(0.0)
    } else {
        value.max(0.0)
    }
}

fn sanitize_unit(value: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        fallback.clamp(0.0, 1.0)
    }
}

fn sanitize_factor(value: f32, fallback: f32) -> f32 {
    if value.is_nan() {
        fallback.clamp(MIN_COLOR_FACTOR, MAX_COLOR_FACTOR)
    } else {
        value.clamp(MIN_COLOR_FACTOR, MAX_COLOR_FACTOR)
    }
}

#[cfg(test)]
mod tests {
    use super::{Backdrop, BackdropQuality};
    use bevy::prelude::*;

    #[test]
    fn backdrop_defaults_are_safe() {
        let backdrop = Backdrop::default();
        assert_eq!(backdrop.blur, 0.0);
        assert_eq!(backdrop.tint, Color::WHITE);
        assert_eq!(backdrop.tint_opacity, 0.0);
        assert_eq!(backdrop.brightness, 1.0);
        assert_eq!(backdrop.saturation, 1.0);
        assert_eq!(backdrop.contrast, 1.0);
        assert_eq!(backdrop.quality, BackdropQuality::Medium);
    }

    #[test]
    fn backdrop_sanitization_clamps_values() {
        let backdrop = Backdrop::new()
            .with_blur(f32::INFINITY)
            .with_tint_opacity(2.0)
            .with_brightness(-5.0)
            .with_saturation(5.0)
            .with_contrast(f32::NAN)
            .sanitized();

        assert_eq!(backdrop.blur, 128.0);
        assert_eq!(backdrop.tint_opacity, 1.0);
        assert_eq!(backdrop.brightness, 0.0);
        assert_eq!(backdrop.saturation, 2.0);
        assert_eq!(backdrop.contrast, 1.0);
    }

    #[test]
    fn backdrop_reports_activity_when_any_processing_is_enabled() {
        assert!(!Backdrop::new().is_active());
        assert!(Backdrop::new().with_blur(8.0).is_active());
        assert!(Backdrop::new().with_tint_opacity(0.2).is_active());
        assert!(Backdrop::new().with_brightness(0.8).is_active());
    }
}
