#![allow(dead_code)] // CPU-side mirror of the mask/clip shader math, for future hit-testing use
use bevy::prelude::*;

use super::{
    sdf::rounded_rect_signed_distance,
    shape::{CornerRadii, Shape},
};

const MAX_CLIP_OPACITY: f32 = 1.0;
const MAX_MASK_OPACITY: f32 = 1.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Clip {
    pub shape: Shape,
    pub opacity: f32,
}

impl Clip {
    pub fn new(shape: Shape) -> Self {
        Self {
            shape,
            opacity: 1.0,
        }
    }

    pub fn rounded_rect(radius: f32) -> Self {
        Self::new(Shape::rounded_rect(radius))
    }

    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = sanitize_unit(opacity, 1.0);
        self
    }

    pub fn sanitized(self) -> Self {
        Self {
            shape: self.shape,
            opacity: sanitize_unit(self.opacity, 1.0).clamp(0.0, MAX_CLIP_OPACITY),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mask {
    pub shape: Shape,
    pub opacity: f32,
}

impl Mask {
    pub fn new(shape: Shape) -> Self {
        Self {
            shape,
            opacity: 1.0,
        }
    }

    pub fn rounded_rect(radius: f32) -> Self {
        Self::new(Shape::rounded_rect(radius))
    }

    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = sanitize_unit(opacity, 1.0);
        self
    }

    pub fn sanitized(self) -> Self {
        Self {
            shape: self.shape,
            opacity: sanitize_unit(self.opacity, 1.0).clamp(0.0, MAX_MASK_OPACITY),
        }
    }
}

pub fn clip_coverage(local: Vec2, shape: Shape, size: Vec2) -> f32 {
    let half_size = size * 0.5;
    match shape {
        Shape::RoundedRect(rect) => {
            let radii = rect.radii.clamped_non_negative();
            let normalized = normalize_mask_radii(radii, size);
            let distance = rounded_rect_signed_distance(local - size * 0.5, half_size, normalized);
            let t = clamp01(distance.max(0.0));
            1.0 - t
        }
    }
}

fn clamp01(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

pub fn mask_coverage(local: Vec2, mask: Mask, size: Vec2) -> f32 {
    let raw = clip_coverage(local, mask.shape, size);
    raw * mask.opacity
}

fn normalize_mask_radii(radii: CornerRadii, size: Vec2) -> CornerRadii {
    let width = size.x.max(0.0);
    let height = size.y.max(0.0);
    if width <= f32::EPSILON || height <= f32::EPSILON {
        return CornerRadii::ZERO;
    }

    let mut out = radii.clamped_non_negative();
    let top = out.top_left + out.top_right;
    let bottom = out.bottom_left + out.bottom_right;
    let left = out.top_left + out.bottom_left;
    let right = out.top_right + out.bottom_right;

    let scale_x = if top > 0.0 || bottom > 0.0 {
        (width / top.max(bottom)).min(1.0)
    } else {
        1.0
    };
    let scale_y = if left > 0.0 || right > 0.0 {
        (height / left.max(right)).min(1.0)
    } else {
        1.0
    };

    let scale = scale_x.min(scale_y).clamp(0.0, 1.0);
    out.top_left *= scale;
    out.top_right *= scale;
    out.bottom_right *= scale;
    out.bottom_left *= scale;
    out
}

fn sanitize_unit(value: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        fallback
    }
}

#[cfg(test)]
mod tests {
    use super::{Clip, Mask};
    use crate::rendering::Shape;

    #[test]
    fn clip_shape_builds_and_keeps_opacity_sane() {
        let clip = Clip::rounded_rect(12.0).with_opacity(0.75);
        assert_eq!(clip.shape, Shape::rounded_rect(12.0));
        assert_eq!(clip.opacity, 0.75);
    }

    #[test]
    fn mask_shape_keeps_transparency_range() {
        let mask = Mask::rounded_rect(10.0).with_opacity(2.5);
        assert_eq!(mask.opacity, 1.0);
    }
}
