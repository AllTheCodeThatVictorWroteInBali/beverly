use bevy::prelude::*;

use super::paint::Paint;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BorderWidths {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl BorderWidths {
    pub const ZERO: Self = Self {
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
        left: 0.0,
    };

    pub fn all(width: f32) -> Self {
        let width = width.max(0.0);
        Self {
            top: width,
            right: width,
            bottom: width,
            left: width,
        }
    }

    pub fn symmetric(vertical: f32, horizontal: f32) -> Self {
        Self {
            top: vertical.max(0.0),
            right: horizontal.max(0.0),
            bottom: vertical.max(0.0),
            left: horizontal.max(0.0),
        }
    }

    pub fn horizontal_vertical(horizontal: f32, vertical: f32) -> Self {
        Self::symmetric(vertical, horizontal)
    }

    pub fn sides(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    pub fn clamped_non_negative(self) -> Self {
        Self {
            top: self.top.max(0.0),
            right: self.right.max(0.0),
            bottom: self.bottom.max(0.0),
            left: self.left.max(0.0),
        }
    }

    pub fn as_vec4(self) -> Vec4 {
        Vec4::new(self.top, self.right, self.bottom, self.left)
    }

    pub fn max_component(self) -> f32 {
        self.top.max(self.right).max(self.bottom).max(self.left)
    }
}

impl Default for BorderWidths {
    fn default() -> Self {
        Self::ZERO
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Border {
    pub width: BorderWidths,
    pub paint: Paint,
}

impl Border {
    pub fn new(width: f32, paint: Paint) -> Self {
        Self {
            width: BorderWidths::all(width),
            paint,
        }
    }

    pub fn per_side(width: BorderWidths, paint: Paint) -> Self {
        Self { width, paint }
    }

    pub fn with_width(mut self, width: BorderWidths) -> Self {
        self.width = width;
        self
    }

    pub fn with_paint(mut self, paint: Paint) -> Self {
        self.paint = paint;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::{Border, BorderWidths};
    use crate::rendering::Paint;
    use bevy::prelude::*;

    #[test]
    fn border_widths_all_sets_uniform_values() {
        let w = BorderWidths::all(2.5);
        assert_eq!(w.top, 2.5);
        assert_eq!(w.right, 2.5);
        assert_eq!(w.bottom, 2.5);
        assert_eq!(w.left, 2.5);
    }

    #[test]
    fn border_new_uses_uniform_widths() {
        let border = Border::new(3.0, Paint::solid(Color::BLACK));
        assert_eq!(border.width, BorderWidths::all(3.0));
    }

    #[test]
    fn border_widths_clamp_negative_values() {
        let w = BorderWidths::sides(-1.0, 2.0, -3.0, 4.0).clamped_non_negative();
        assert_eq!(w.top, 0.0);
        assert_eq!(w.right, 2.0);
        assert_eq!(w.bottom, 0.0);
        assert_eq!(w.left, 4.0);
    }
}
