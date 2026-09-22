use bevy::prelude::*;

use super::{effect::OuterGlow, paint::Paint};

const MAX_FOCUS_WIDTH: f32 = 256.0;
const MAX_FOCUS_OFFSET: f32 = 512.0;

/// Semantic decoration primitives rendered independently from a surface's fill/border.
#[derive(Clone, Debug, PartialEq)]
pub enum Decoration {
    FocusRing(FocusRing),
    Selection(SelectionDecoration),
    Validation(ValidationDecoration),
}

/// Surface-attached decoration slots.
///
/// Focus is implemented now, while selection/validation slots establish
/// extension points for later milestones.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Decorations {
    pub focus_ring: Option<FocusRing>,
    pub selection: Option<SelectionDecoration>,
    pub validation: Option<ValidationDecoration>,
}

impl Decorations {
    pub fn with(mut self, decoration: Decoration) -> Self {
        self.apply(decoration);
        self
    }

    pub fn apply(&mut self, decoration: Decoration) {
        match decoration {
            Decoration::FocusRing(value) => self.focus_ring = Some(value),
            Decoration::Selection(value) => self.selection = Some(value),
            Decoration::Validation(value) => self.validation = Some(value),
        }
    }

    pub fn clear_focus_ring(mut self) -> Self {
        self.focus_ring = None;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FocusRingPlacement {
    /// Ring sits fully outside the surface boundary.
    #[default]
    Outside,
    /// Ring straddles the boundary centerline.
    Center,
    /// Ring sits fully inside the surface boundary.
    Inside,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FocusRingLayer {
    pub width: f32,
    pub offset: f32,
    pub opacity: f32,
    pub paint: Paint,
}

impl FocusRingLayer {
    pub fn new(width: f32, offset: f32, paint: impl Into<Paint>) -> Self {
        Self {
            width,
            offset,
            opacity: 1.0,
            paint: paint.into(),
        }
    }

    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
        self
    }

    pub fn sanitized(self) -> Self {
        Self {
            width: sanitize_non_negative(self.width, 0.0).min(MAX_FOCUS_WIDTH),
            offset: sanitize_non_negative(self.offset, 0.0).min(MAX_FOCUS_OFFSET),
            opacity: sanitize_unit(self.opacity, 1.0),
            paint: self.paint,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FocusRing {
    pub placement: FocusRingPlacement,
    pub primary: FocusRingLayer,
    pub secondary: Option<FocusRingLayer>,
    pub glow: Option<OuterGlow>,
}

impl FocusRing {
    pub fn new(primary: FocusRingLayer) -> Self {
        Self {
            placement: FocusRingPlacement::Outside,
            primary,
            secondary: None,
            glow: None,
        }
    }

    pub fn outside(width: f32, offset: f32, paint: impl Into<Paint>) -> Self {
        Self::new(FocusRingLayer::new(width, offset, paint))
    }

    pub fn with_placement(mut self, placement: FocusRingPlacement) -> Self {
        self.placement = placement;
        self
    }

    pub fn with_secondary(mut self, layer: FocusRingLayer) -> Self {
        self.secondary = Some(layer);
        self
    }

    pub fn with_glow(mut self, glow: OuterGlow) -> Self {
        self.glow = Some(glow);
        self
    }

    pub fn sanitized(self) -> Self {
        Self {
            placement: self.placement,
            primary: self.primary.sanitized(),
            secondary: self.secondary.map(FocusRingLayer::sanitized),
            glow: self.glow.map(OuterGlow::sanitized),
        }
    }

    pub fn max_outer_extent(&self) -> f32 {
        let focus = self.clone().sanitized();

        let primary = layer_outer_extent(focus.placement, &focus.primary);
        let secondary = focus
            .secondary
            .as_ref()
            .map(|layer| layer_outer_extent(focus.placement, layer))
            .unwrap_or(0.0);

        let mut outer = primary.max(secondary);

        if let Some(glow) = focus.glow {
            let blur_extent = shadow_falloff_extent(glow.blur, glow.falloff);
            outer += glow.spread.max(0.0) + blur_extent + 1.0;
        }

        outer.max(0.0)
    }
}

impl Default for FocusRing {
    fn default() -> Self {
        Self::outside(2.0, 2.0, Color::srgb(0.23, 0.51, 0.96))
    }
}

/// Reserved semantic decoration slot for future selected-state visuals.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct SelectionDecoration {
    pub enabled: bool,
}

/// Reserved semantic decoration slot for future validation semantics.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ValidationDecoration {
    pub enabled: bool,
}

fn layer_outer_extent(placement: FocusRingPlacement, layer: &FocusRingLayer) -> f32 {
    let width = layer.width.max(0.0);
    let offset = layer.offset.max(0.0);

    match placement {
        FocusRingPlacement::Outside => offset + width,
        FocusRingPlacement::Center => (offset + width * 0.5).max(0.0),
        FocusRingPlacement::Inside => 0.0,
    }
}

fn shadow_falloff_extent(blur: f32, falloff: super::effect::ShadowFalloff) -> f32 {
    let blur = blur.max(0.0);
    match falloff {
        super::effect::ShadowFalloff::Linear => blur,
        super::effect::ShadowFalloff::Smooth => blur * 1.25,
        super::effect::ShadowFalloff::Gaussian => blur * 3.0,
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

#[cfg(test)]
mod tests {
    use super::{Decorations, FocusRing, FocusRingLayer, FocusRingPlacement};
    use crate::rendering::{Decoration, Paint};
    use bevy::prelude::*;

    #[test]
    fn focus_ring_layer_sanitizes_values() {
        let layer = FocusRingLayer::new(-4.0, -8.0, Paint::solid(Color::WHITE)).with_opacity(8.0);
        let sanitized = layer.sanitized();

        assert_eq!(sanitized.width, 0.0);
        assert_eq!(sanitized.offset, 0.0);
        assert_eq!(sanitized.opacity, 1.0);
    }

    #[test]
    fn focus_ring_extent_respects_placement() {
        let outside = FocusRing::outside(2.0, 3.0, Color::WHITE);
        let center = FocusRing::outside(2.0, 3.0, Color::WHITE).with_placement(FocusRingPlacement::Center);
        let inside = FocusRing::outside(2.0, 3.0, Color::WHITE).with_placement(FocusRingPlacement::Inside);

        assert_eq!(outside.max_outer_extent(), 5.0);
        assert_eq!(center.max_outer_extent(), 4.0);
        assert_eq!(inside.max_outer_extent(), 0.0);
    }

    #[test]
    fn decorations_apply_focus_ring_semantically() {
        let focus = FocusRing::outside(2.0, 2.0, Color::WHITE);
        let decorations = Decorations::default().with(Decoration::FocusRing(focus.clone()));
        assert_eq!(decorations.focus_ring, Some(focus));
    }
}
