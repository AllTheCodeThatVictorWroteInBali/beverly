use bevy::prelude::*;

const MAX_EFFECT_BLUR: f32 = 4096.0;
const MAX_EFFECT_SPREAD: f32 = 4096.0;
const MAX_EFFECT_OFFSET: f32 = 8192.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShadowFalloff {
    Linear,
    Smooth,
    Gaussian,
}

impl Default for ShadowFalloff {
    fn default() -> Self {
        Self::Smooth
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OuterShadow {
    pub color: Color,
    pub offset: Vec2,
    pub blur: f32,
    pub spread: f32,
    pub opacity: f32,
    pub falloff: ShadowFalloff,
}

impl OuterShadow {
    pub fn new(color: Color) -> Self {
        Self {
            color,
            ..Self::default()
        }
    }

    /// Bootstrap `.shadow-sm`-style preset: `0 2px 4px rgba(0,0,0,.075)`.
    pub fn small(color: Color) -> Self {
        Self::new(color)
            .with_offset(Vec2::new(0.0, 2.0))
            .with_blur(4.0)
            .with_opacity(0.075)
            .with_falloff(ShadowFalloff::Smooth)
    }

    /// Bootstrap `.shadow`-style preset: `0 8px 16px rgba(0,0,0,.15)`.
    pub fn regular(color: Color) -> Self {
        Self::new(color)
            .with_offset(Vec2::new(0.0, 8.0))
            .with_blur(16.0)
            .with_opacity(0.15)
            .with_falloff(ShadowFalloff::Smooth)
    }

    /// Bootstrap `.shadow-lg`-style preset: `0 16px 48px rgba(0,0,0,.175)`.
    pub fn large(color: Color) -> Self {
        Self::new(color)
            .with_offset(Vec2::new(0.0, 16.0))
            .with_blur(48.0)
            .with_opacity(0.175)
            .with_falloff(ShadowFalloff::Smooth)
    }

    pub fn with_offset(mut self, offset: Vec2) -> Self {
        self.offset = offset;
        self
    }

    pub fn with_blur(mut self, blur: f32) -> Self {
        self.blur = blur;
        self
    }

    pub fn with_spread(mut self, spread: f32) -> Self {
        self.spread = spread;
        self
    }

    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
        self
    }

    pub fn with_falloff(mut self, falloff: ShadowFalloff) -> Self {
        self.falloff = falloff;
        self
    }

    pub fn sanitized(self) -> Self {
        Self {
            color: self.color,
            offset: sanitize_vec2(self.offset, Vec2::ZERO)
                .clamp(Vec2::splat(-MAX_EFFECT_OFFSET), Vec2::splat(MAX_EFFECT_OFFSET)),
            blur: sanitize_non_negative(self.blur, 0.0).min(MAX_EFFECT_BLUR),
            spread: sanitize_finite(self.spread, 0.0).clamp(-MAX_EFFECT_SPREAD, MAX_EFFECT_SPREAD),
            opacity: sanitize_unit(self.opacity, 0.0),
            falloff: self.falloff,
        }
    }
}

impl Default for OuterShadow {
    fn default() -> Self {
        Self {
            color: Color::BLACK,
            offset: Vec2::ZERO,
            blur: 12.0,
            spread: 0.0,
            opacity: 0.0,
            falloff: ShadowFalloff::Smooth,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OuterGlow {
    pub color: Color,
    pub blur: f32,
    pub spread: f32,
    pub opacity: f32,
    pub falloff: ShadowFalloff,
}

impl OuterGlow {
    pub fn new(color: Color) -> Self {
        Self {
            color,
            ..Self::default()
        }
    }

    pub fn with_blur(mut self, blur: f32) -> Self {
        self.blur = blur;
        self
    }

    pub fn with_spread(mut self, spread: f32) -> Self {
        self.spread = spread;
        self
    }

    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
        self
    }

    pub fn with_falloff(mut self, falloff: ShadowFalloff) -> Self {
        self.falloff = falloff;
        self
    }

    pub fn sanitized(self) -> Self {
        Self {
            color: self.color,
            blur: sanitize_non_negative(self.blur, 0.0).min(MAX_EFFECT_BLUR),
            spread: sanitize_finite(self.spread, 0.0).clamp(-MAX_EFFECT_SPREAD, MAX_EFFECT_SPREAD),
            opacity: sanitize_unit(self.opacity, 0.0),
            falloff: self.falloff,
        }
    }
}

impl Default for OuterGlow {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            blur: 10.0,
            spread: 0.0,
            opacity: 0.0,
            falloff: ShadowFalloff::Gaussian,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InnerShadow {
    pub color: Color,
    pub offset: Vec2,
    pub blur: f32,
    pub spread: f32,
    pub opacity: f32,
    pub falloff: ShadowFalloff,
}

impl InnerShadow {
    pub fn new(color: Color) -> Self {
        Self {
            color,
            ..Self::default()
        }
    }

    /// Inset counterpart of [`OuterShadow::small`]: `inset 0 1px 2px rgba(0,0,0,.075)`.
    pub fn small(color: Color) -> Self {
        Self::new(color)
            .with_offset(Vec2::new(0.0, 1.0))
            .with_blur(2.0)
            .with_opacity(0.075)
            .with_falloff(ShadowFalloff::Smooth)
    }

    /// Inset counterpart of [`OuterShadow::regular`]: `inset 0 2px 4px rgba(0,0,0,.125)`.
    pub fn regular(color: Color) -> Self {
        Self::new(color)
            .with_offset(Vec2::new(0.0, 2.0))
            .with_blur(4.0)
            .with_opacity(0.125)
            .with_falloff(ShadowFalloff::Smooth)
    }

    /// Inset counterpart of [`OuterShadow::large`]: `inset 0 4px 8px rgba(0,0,0,.175)`.
    pub fn large(color: Color) -> Self {
        Self::new(color)
            .with_offset(Vec2::new(0.0, 4.0))
            .with_blur(8.0)
            .with_opacity(0.175)
            .with_falloff(ShadowFalloff::Smooth)
    }

    pub fn with_offset(mut self, offset: Vec2) -> Self {
        self.offset = offset;
        self
    }

    pub fn with_blur(mut self, blur: f32) -> Self {
        self.blur = blur;
        self
    }

    pub fn with_spread(mut self, spread: f32) -> Self {
        self.spread = spread;
        self
    }

    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
        self
    }

    pub fn with_falloff(mut self, falloff: ShadowFalloff) -> Self {
        self.falloff = falloff;
        self
    }

    pub fn sanitized(self) -> Self {
        Self {
            color: self.color,
            offset: sanitize_vec2(self.offset, Vec2::ZERO)
                .clamp(Vec2::splat(-MAX_EFFECT_OFFSET), Vec2::splat(MAX_EFFECT_OFFSET)),
            blur: sanitize_non_negative(self.blur, 0.0).min(MAX_EFFECT_BLUR),
            spread: sanitize_finite(self.spread, 0.0).clamp(-MAX_EFFECT_SPREAD, MAX_EFFECT_SPREAD),
            opacity: sanitize_unit(self.opacity, 0.0),
            falloff: self.falloff,
        }
    }
}

impl Default for InnerShadow {
    fn default() -> Self {
        Self {
            color: Color::BLACK,
            offset: Vec2::ZERO,
            blur: 8.0,
            spread: 0.0,
            opacity: 0.0,
            falloff: ShadowFalloff::Smooth,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Effect {
    OuterShadow(OuterShadow),
    OuterGlow(OuterGlow),
    InnerShadow(InnerShadow),
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Effects {
    pub outer_shadow: Option<OuterShadow>,
    pub outer_glow: Option<OuterGlow>,
    pub inner_shadow: Option<InnerShadow>,
}

impl Effects {
    pub fn with_effect(mut self, effect: Effect) -> Self {
        self.apply(effect);
        self
    }

    pub fn apply(&mut self, effect: Effect) {
        match effect {
            Effect::OuterShadow(shadow) => self.outer_shadow = Some(shadow),
            Effect::OuterGlow(glow) => self.outer_glow = Some(glow),
            Effect::InnerShadow(shadow) => self.inner_shadow = Some(shadow),
        }
    }

    pub fn with_outer_shadow(mut self, shadow: OuterShadow) -> Self {
        self.outer_shadow = Some(shadow);
        self
    }

    pub fn with_outer_glow(mut self, glow: OuterGlow) -> Self {
        self.outer_glow = Some(glow);
        self
    }

    pub fn clear_outer_shadow(mut self) -> Self {
        self.outer_shadow = None;
        self
    }

    pub fn clear_outer_glow(mut self) -> Self {
        self.outer_glow = None;
        self
    }

    pub fn with_inner_shadow(mut self, shadow: InnerShadow) -> Self {
        self.inner_shadow = Some(shadow);
        self
    }

    pub fn clear_inner_shadow(mut self) -> Self {
        self.inner_shadow = None;
        self
    }
}

impl FromIterator<Effect> for Effects {
    fn from_iter<T: IntoIterator<Item = Effect>>(iter: T) -> Self {
        let mut effects = Self::default();
        for effect in iter {
            effects.apply(effect);
        }
        effects
    }
}

fn sanitize_finite(value: f32, fallback: f32) -> f32 {
    if value.is_nan() {
        fallback
    } else {
        value
    }
}

fn sanitize_non_negative(value: f32, fallback: f32) -> f32 {
    sanitize_finite(value, fallback).max(0.0)
}

fn sanitize_unit(value: f32, fallback: f32) -> f32 {
    sanitize_finite(value, fallback).clamp(0.0, 1.0)
}

fn sanitize_vec2(value: Vec2, fallback: Vec2) -> Vec2 {
    Vec2::new(
        sanitize_finite(value.x, fallback.x),
        sanitize_finite(value.y, fallback.y),
    )
}

#[cfg(test)]
mod tests {
    use super::{Effect, Effects, InnerShadow, OuterGlow, OuterShadow, ShadowFalloff};
    use bevy::prelude::*;

    #[test]
    fn outer_shadow_default_is_safe_and_invisible() {
        let shadow = OuterShadow::default();
        assert_eq!(shadow.opacity, 0.0);
        assert!(shadow.blur >= 0.0);
        assert_eq!(shadow.spread, 0.0);
        assert_eq!(shadow.offset, Vec2::ZERO);
    }

    #[test]
    fn outer_glow_default_is_safe_and_invisible() {
        let glow = OuterGlow::default();
        assert_eq!(glow.opacity, 0.0);
        assert!(glow.blur >= 0.0);
        assert_eq!(glow.spread, 0.0);
    }

    #[test]
    fn inner_shadow_default_is_safe_and_invisible() {
        let shadow = InnerShadow::default();
        assert_eq!(shadow.opacity, 0.0);
        assert!(shadow.blur >= 0.0);
        assert_eq!(shadow.spread, 0.0);
        assert_eq!(shadow.offset, Vec2::ZERO);
    }

    #[test]
    fn outer_shadow_sanitizes_invalid_values() {
        let shadow = OuterShadow::new(Color::BLACK)
            .with_offset(Vec2::new(f32::NAN, f32::INFINITY))
            .with_blur(f32::NEG_INFINITY)
            .with_spread(f32::INFINITY)
            .with_opacity(8.0)
            .with_falloff(ShadowFalloff::Gaussian)
            .sanitized();

        assert_eq!(shadow.offset, Vec2::new(0.0, super::MAX_EFFECT_OFFSET));
        assert_eq!(shadow.blur, 0.0);
        assert_eq!(shadow.spread, super::MAX_EFFECT_SPREAD);
        assert_eq!(shadow.opacity, 1.0);
        assert_eq!(shadow.falloff, ShadowFalloff::Gaussian);
    }

    #[test]
    fn outer_glow_sanitizes_invalid_values() {
        let glow = OuterGlow::new(Color::WHITE)
            .with_blur(f32::NAN)
            .with_spread(f32::NEG_INFINITY)
            .with_opacity(-0.5)
            .sanitized();

        assert_eq!(glow.blur, 0.0);
        assert_eq!(glow.spread, -super::MAX_EFFECT_SPREAD);
        assert_eq!(glow.opacity, 0.0);
    }

    #[test]
    fn effects_builder_sets_expected_fields() {
        let effects = Effects::default()
            .with_outer_shadow(OuterShadow::new(Color::BLACK).with_opacity(0.2))
            .with_outer_glow(OuterGlow::new(Color::WHITE).with_opacity(0.4))
            .with_inner_shadow(InnerShadow::new(Color::BLACK).with_opacity(0.3));

        assert!(effects.outer_shadow.is_some());
        assert!(effects.outer_glow.is_some());
        assert!(effects.inner_shadow.is_some());
    }

    #[test]
    fn inner_shadow_sanitizes_invalid_values() {
        let shadow = InnerShadow::new(Color::BLACK)
            .with_offset(Vec2::new(f32::NEG_INFINITY, f32::NAN))
            .with_blur(-10.0)
            .with_spread(f32::INFINITY)
            .with_opacity(-0.2)
            .sanitized();

        assert_eq!(shadow.offset, Vec2::new(-super::MAX_EFFECT_OFFSET, 0.0));
        assert_eq!(shadow.blur, 0.0);
        assert_eq!(shadow.spread, super::MAX_EFFECT_SPREAD);
        assert_eq!(shadow.opacity, 0.0);
    }

    #[test]
    fn sanitize_clamps_extreme_finite_values() {
        let shadow = InnerShadow::new(Color::BLACK)
            .with_offset(Vec2::new(999_999.0, -999_999.0))
            .with_blur(999_999.0)
            .with_spread(-999_999.0)
            .with_opacity(999_999.0)
            .sanitized();

        assert_eq!(shadow.offset, Vec2::new(super::MAX_EFFECT_OFFSET, -super::MAX_EFFECT_OFFSET));
        assert_eq!(shadow.blur, super::MAX_EFFECT_BLUR);
        assert_eq!(shadow.spread, -super::MAX_EFFECT_SPREAD);
        assert_eq!(shadow.opacity, 1.0);
    }

    #[test]
    fn effects_from_iter_is_semantic_and_last_wins_per_variant() {
        let effects = vec![
            Effect::OuterShadow(OuterShadow::new(Color::BLACK).with_opacity(0.1)),
            Effect::InnerShadow(InnerShadow::new(Color::BLACK).with_opacity(0.2)),
            Effect::OuterGlow(OuterGlow::new(Color::WHITE).with_opacity(0.3)),
            Effect::InnerShadow(InnerShadow::new(Color::WHITE).with_opacity(0.6)),
        ]
        .into_iter()
        .collect::<Effects>();

        assert_eq!(effects.outer_shadow.unwrap().opacity, 0.1);
        assert_eq!(effects.outer_glow.unwrap().opacity, 0.3);
        assert_eq!(effects.inner_shadow.unwrap().opacity, 0.6);
    }
}