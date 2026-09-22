use bevy::prelude::*;

use super::{
    backdrop::Backdrop,
    border::Border,
    decoration::{Decorations, FocusRing},
    effect::{Effects, InnerShadow, OuterGlow, OuterShadow},
    mask::{Clip, Mask},
    noise::Noise,
    paint::Paint,
    shape::Shape,
};

/// Visual description of a UI surface.
///
/// This separates geometry (shape) from appearance (paint), so future
/// additions like borders, shadows, and gradients can layer on top without
/// rewriting component-level APIs. Semantic interaction indicators live in
/// `decorations`, independent from border styling.
#[derive(Component, Clone, Debug, PartialEq)]
pub struct Surface {
    pub shape: Shape,
    pub fill: Paint,
    pub border: Option<Border>,
    pub decorations: Decorations,
    pub effects: Effects,
    pub noise: Option<Noise>,
    pub clip: Option<Clip>,
    pub mask: Option<Mask>,
    pub backdrop: Option<Backdrop>,
}

impl Surface {
    pub fn new(shape: Shape, fill: Paint) -> Self {
        Self {
            shape,
            fill,
            border: None,
            decorations: Decorations::default(),
            effects: Effects::default(),
            noise: None,
            clip: None,
            mask: None,
            backdrop: None,
        }
    }

    pub fn rounded_rect_fill(radius: f32, fill: impl Into<Paint>) -> Self {
        Self {
            shape: Shape::rounded_rect(radius),
            fill: fill.into(),
            border: None,
            decorations: Decorations::default(),
            effects: Effects::default(),
            noise: None,
            clip: None,
            mask: None,
            backdrop: None,
        }
    }

    pub fn rounded_rect_border(
        radius: f32,
        fill: impl Into<Paint>,
        border_width: f32,
        border: impl Into<Paint>,
    ) -> Self {
        Self {
            shape: Shape::rounded_rect(radius),
            fill: fill.into(),
            border: Some(Border::new(border_width, border.into())),
            decorations: Decorations::default(),
            effects: Effects::default(),
            noise: None,
            clip: None,
            mask: None,
            backdrop: None,
        }
    }

    pub fn with_shape(mut self, shape: Shape) -> Self {
        self.shape = shape;
        self
    }

    pub fn fill(mut self, paint: impl Into<Paint>) -> Self {
        self.fill = paint.into();
        self
    }

    pub fn border(mut self, border: Border) -> Self {
        self.border = Some(border);
        self
    }

    pub fn clear_border(mut self) -> Self {
        self.border = None;
        self
    }

    pub fn with_decorations(mut self, decorations: Decorations) -> Self {
        self.decorations = decorations;
        self
    }

    pub fn with_focus_ring(mut self, focus_ring: FocusRing) -> Self {
        self.decorations.focus_ring = Some(focus_ring);
        self
    }

    pub fn clear_focus_ring(mut self) -> Self {
        self.decorations.focus_ring = None;
        self
    }

    pub fn with_effects(mut self, effects: Effects) -> Self {
        self.effects = effects;
        self
    }

    pub fn with_noise(mut self, noise: Noise) -> Self {
        self.noise = Some(noise);
        self
    }

    pub fn clear_noise(mut self) -> Self {
        self.noise = None;
        self
    }

    pub fn with_clip(mut self, clip: Clip) -> Self {
        self.clip = Some(clip);
        self
    }

    pub fn clear_clip(mut self) -> Self {
        self.clip = None;
        self
    }

    pub fn with_mask(mut self, mask: Mask) -> Self {
        self.mask = Some(mask);
        self
    }

    pub fn clear_mask(mut self) -> Self {
        self.mask = None;
        self
    }

    pub fn with_backdrop(mut self, backdrop: Backdrop) -> Self {
        self.backdrop = Some(backdrop);
        self
    }

    pub fn clear_backdrop(mut self) -> Self {
        self.backdrop = None;
        self
    }

    pub fn outer_shadow(mut self, shadow: OuterShadow) -> Self {
        self.effects.outer_shadow = Some(shadow);
        self
    }

    pub fn clear_outer_shadow(mut self) -> Self {
        self.effects.outer_shadow = None;
        self
    }

    pub fn outer_glow(mut self, glow: OuterGlow) -> Self {
        self.effects.outer_glow = Some(glow);
        self
    }

    pub fn clear_outer_glow(mut self) -> Self {
        self.effects.outer_glow = None;
        self
    }

    pub fn inner_shadow(mut self, shadow: InnerShadow) -> Self {
        self.effects.inner_shadow = Some(shadow);
        self
    }

    pub fn clear_inner_shadow(mut self) -> Self {
        self.effects.inner_shadow = None;
        self
    }

    pub fn uniform_border(mut self, width: f32, paint: impl Into<Paint>) -> Self {
        self.border = Some(Border::new(width, paint.into()));
        self
    }
}
