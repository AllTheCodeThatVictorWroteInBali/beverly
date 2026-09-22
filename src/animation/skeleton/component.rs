use bevy::prelude::*;

use super::{SkeletonDirection, SkeletonTextLines};

/// A decorative, non-interactive loading silhouette rendered by native Surface.
///
/// `new()` leaves the caller's Node dimensions untouched. `text()` supplies a
/// typography-derived height only when the caller has not provided one.
/// `enabled(false)` disables shimmer, not layout or visibility. Despawn the
/// loading group when content arrives; skeletons are not progress indicators.
#[derive(Component, Clone, Debug, PartialEq)]
#[require(Node, Pickable::IGNORE, SkeletonLayoutState, crate::rendering::SharedSurfaceMaterial)]
pub struct Skeleton {
    pub(crate) width: Option<Val>,
    pub(crate) height: Option<Val>,
    pub(crate) text: bool,
    pub(crate) radius: f32,
    pub(crate) enabled: bool,
    pub(crate) direction: SkeletonDirection,
    pub(crate) duration: f32,
    pub(crate) phase: f32,
    pub(crate) highlight_width: f32,
    pub(crate) highlight_softness: f32,
    pub(crate) highlight_intensity: f32,
    pub(crate) base_color: Option<Color>,
    pub(crate) highlight_color: Option<Color>,
}

/// Tracks only typography-owned height, never an animation clock.
#[derive(Component, Default)]
pub(crate) struct SkeletonLayoutState {
    pub text_height: Option<Val>,
}

impl Default for Skeleton {
    fn default() -> Self {
        Self {
            width: None,
            height: None,
            text: false,
            radius: 6.0,
            enabled: true,
            direction: SkeletonDirection::LeftToRight,
            duration: 1.5,
            phase: 0.0,
            highlight_width: 0.22,
            highlight_softness: 0.8,
            highlight_intensity: 0.65,
            base_color: None,
            highlight_color: None,
        }
    }
}

impl Skeleton {
    pub fn new() -> Self { Self::default() }

    pub fn text() -> Self {
        Self { text: true, radius: 3.0, ..Self::default() }
    }

    /// Fixed-size circle, implemented with canonical rounded-rectangle geometry.
    pub fn circle(size: f32) -> Self {
        let size = finite_nonnegative(size, 0.0);
        Self::new().width(Val::Px(size)).height(Val::Px(size)).radius(size * 0.5)
    }

    pub fn text_lines(count: usize) -> SkeletonTextLines { SkeletonTextLines::new(count) }

    /// Override only Node.width; percent and auto values retain native semantics.
    pub fn width(mut self, width: Val) -> Self { self.width = Some(width); self }
    /// Override only Node.height, including typography-derived text height.
    pub fn height(mut self, height: Val) -> Self { self.height = Some(height); self }
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = finite_nonnegative(radius, 6.0); self
    }
    pub fn enabled(mut self, enabled: bool) -> Self { self.enabled = enabled; self }
    pub fn direction(mut self, direction: SkeletonDirection) -> Self { self.direction = direction; self }
    /// Seconds per sweep. Invalid values are normalized by Shimmer::sanitized.
    pub fn duration(mut self, seconds: f32) -> Self { self.duration = seconds; self }
    /// Normalized cycle offset, not an independent CPU timer.
    pub fn phase(mut self, phase: f32) -> Self { self.phase = phase; self }
    pub fn highlight_width(mut self, width: f32) -> Self { self.highlight_width = width; self }
    pub fn highlight_softness(mut self, softness: f32) -> Self { self.highlight_softness = softness; self }
    pub fn highlight_intensity(mut self, intensity: f32) -> Self { self.highlight_intensity = intensity; self }
    pub fn color(self, color: Color) -> Self { self.base_color(color) }
    pub fn base_color(mut self, color: Color) -> Self { self.base_color = Some(color); self }
    pub fn highlight_color(mut self, color: Color) -> Self { self.highlight_color = Some(color); self }
    /// Remove both overrides and resume following semantic theme tokens.
    pub fn theme_colors(mut self) -> Self { self.base_color = None; self.highlight_color = None; self }
}

pub(crate) fn finite_nonnegative(value: f32, fallback: f32) -> f32 {
    if value.is_finite() { value.max(0.0) } else { fallback }
}