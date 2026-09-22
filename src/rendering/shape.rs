use bevy::prelude::*;

/// Corner radii in logical UI pixels.
///
/// Ordering is clockwise starting at the top-left corner.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CornerRadii {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}

impl CornerRadii {
    pub const ZERO: Self = Self {
        top_left: 0.0,
        top_right: 0.0,
        bottom_right: 0.0,
        bottom_left: 0.0,
    };

    pub fn new(radius: f32) -> Self {
        let radius = radius.max(0.0);
        Self {
            top_left: radius,
            top_right: radius,
            bottom_right: radius,
            bottom_left: radius,
        }
    }

    pub fn corners(top_left: f32, top_right: f32, bottom_right: f32, bottom_left: f32) -> Self {
        Self {
            top_left,
            top_right,
            bottom_right,
            bottom_left,
        }
    }

    pub fn clamped_non_negative(self) -> Self {
        Self {
            top_left: self.top_left.max(0.0),
            top_right: self.top_right.max(0.0),
            bottom_right: self.bottom_right.max(0.0),
            bottom_left: self.bottom_left.max(0.0),
        }
    }

    pub fn as_vec4(self) -> Vec4 {
        Vec4::new(
            self.top_left,
            self.top_right,
            self.bottom_right,
            self.bottom_left,
        )
    }
}

impl Default for CornerRadii {
    fn default() -> Self {
        Self::ZERO
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RoundedRect {
    pub radii: CornerRadii,
}

impl RoundedRect {
    pub fn new(radius: f32) -> Self {
        Self {
            radii: CornerRadii::new(radius),
        }
    }

    pub fn corners(top_left: f32, top_right: f32, bottom_right: f32, bottom_left: f32) -> Self {
        Self {
            radii: CornerRadii::corners(top_left, top_right, bottom_right, bottom_left),
        }
    }
}

impl Default for RoundedRect {
    fn default() -> Self {
        Self::new(0.0)
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub enum Shape {
    RoundedRect(RoundedRect),
}

impl Shape {
    pub fn rounded_rect(radius: f32) -> Self {
        Self::RoundedRect(RoundedRect::new(radius))
    }

    pub fn rounded_rect_corners(
        top_left: f32,
        top_right: f32,
        bottom_right: f32,
        bottom_left: f32,
    ) -> Self {
        Self::RoundedRect(RoundedRect::corners(
            top_left,
            top_right,
            bottom_right,
            bottom_left,
        ))
    }
}

impl Default for Shape {
    fn default() -> Self {
        Self::RoundedRect(RoundedRect::default())
    }
}

#[cfg(test)]
mod tests {
    use super::{CornerRadii, RoundedRect};

    #[test]
    fn rounded_rect_new_sets_uniform_radius() {
        let rect = RoundedRect::new(12.0);
        assert_eq!(rect.radii, CornerRadii::new(12.0));
    }

    #[test]
    fn rounded_rect_corners_keeps_corner_order() {
        let rect = RoundedRect::corners(1.0, 2.0, 3.0, 4.0);
        assert_eq!(rect.radii.top_left, 1.0);
        assert_eq!(rect.radii.top_right, 2.0);
        assert_eq!(rect.radii.bottom_right, 3.0);
        assert_eq!(rect.radii.bottom_left, 4.0);
    }
}
