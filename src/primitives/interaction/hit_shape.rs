use bevy::prelude::*;

use crate::rendering::{CornerRadii, Shape};

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct HitSlop {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl HitSlop {
    pub fn all(v: f32) -> Self {
        let v = v.max(0.0);
        Self {
            top: v,
            right: v,
            bottom: v,
            left: v,
        }
    }
}

impl Default for HitSlop {
    fn default() -> Self {
        Self::all(0.0)
    }
}

#[derive(Component, Clone, Debug, PartialEq)]
pub enum HitShape {
    LayoutRect,
    RoundedRect(CornerRadii),
    Circle,
    CustomRect(Rect),
}

impl HitShape {
    pub fn from_render_shape(shape: Shape) -> Self {
        match shape {
            Shape::RoundedRect(rounded) => Self::RoundedRect(rounded.radii),
        }
    }

    pub fn contains_local(self, local: Vec2, size: Vec2, slop: HitSlop) -> bool {
        let expanded = Rect::from_corners(
            Vec2::new(-slop.left, -slop.top),
            Vec2::new(size.x + slop.right, size.y + slop.bottom),
        );

        if !expanded.contains(local) {
            return false;
        }

        match self {
            Self::LayoutRect => true,
            Self::Circle => {
                let center = size * 0.5;
                let radius = size.min_element() * 0.5 + slop.top.max(slop.left).max(slop.right).max(slop.bottom);
                local.distance_squared(center) <= radius * radius
            }
            Self::CustomRect(rect) => rect.contains(local),
            Self::RoundedRect(radii) => contains_rounded_rect(local, size, radii, slop),
        }
    }
}

fn contains_rounded_rect(local: Vec2, size: Vec2, radii: CornerRadii, slop: HitSlop) -> bool {
    let slop_max = slop.top.max(slop.right).max(slop.bottom).max(slop.left);
    let width = (size.x + slop.left + slop.right).max(0.0);
    let height = (size.y + slop.top + slop.bottom).max(0.0);
    let x = local.x + slop.left;
    let y = local.y + slop.top;

    if x < 0.0 || y < 0.0 || x > width || y > height {
        return false;
    }

    let tl = (radii.top_left + slop_max).max(0.0).min(width * 0.5).min(height * 0.5);
    let tr = (radii.top_right + slop_max).max(0.0).min(width * 0.5).min(height * 0.5);
    let br = (radii.bottom_right + slop_max).max(0.0).min(width * 0.5).min(height * 0.5);
    let bl = (radii.bottom_left + slop_max).max(0.0).min(width * 0.5).min(height * 0.5);

    if x >= tl && x <= width - tr {
        return true;
    }
    if y >= tl && y <= height - bl {
        return true;
    }

    if x < tl && y < tl {
        let c = Vec2::new(tl, tl);
        return (Vec2::new(x, y) - c).length_squared() <= tl * tl;
    }
    if x > width - tr && y < tr {
        let c = Vec2::new(width - tr, tr);
        return (Vec2::new(x, y) - c).length_squared() <= tr * tr;
    }
    if x > width - br && y > height - br {
        let c = Vec2::new(width - br, height - br);
        return (Vec2::new(x, y) - c).length_squared() <= br * br;
    }
    if x < bl && y > height - bl {
        let c = Vec2::new(bl, height - bl);
        return (Vec2::new(x, y) - c).length_squared() <= bl * bl;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounded_rect_contains_center() {
        let shape = HitShape::RoundedRect(CornerRadii::new(12.0));
        assert!(shape.contains_local(Vec2::new(50.0, 20.0), Vec2::new(100.0, 40.0), HitSlop::default()));
    }

    #[test]
    fn circle_rejects_outside_point() {
        let shape = HitShape::Circle;
        assert!(!shape.contains_local(Vec2::new(50.0, 0.0), Vec2::new(40.0, 40.0), HitSlop::default()));
    }

    #[test]
    fn hit_slop_expands_target() {
        let shape = HitShape::LayoutRect;
        assert!(shape.contains_local(Vec2::new(-8.0, 20.0), Vec2::new(40.0, 40.0), HitSlop::all(10.0)));
    }
}
