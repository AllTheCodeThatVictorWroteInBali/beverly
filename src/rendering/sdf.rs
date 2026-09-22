use bevy::prelude::*;

use super::{border::BorderWidths, shape::CornerRadii};

// One million logical pixels leaves ample room for offscreen UI while keeping
// pair sums and subsequent SDF arithmetic far below f32 overflow.
const MAX_GEOMETRY_EXTENT: f32 = 1_048_576.0;

fn bounded_extent(value: f32) -> f32 {
    // max maps NaN/negative values to zero; min saturates positive infinity.
    value.max(0.0).min(MAX_GEOMETRY_EXTENT)
}

/// Normalizes corner radii using the same proportional scaling rule used by
/// CSS border radius resolution.
///
/// Input/Output are in logical UI pixels.
/// NaN/negative inputs become zero; positive infinity and extreme extents are
/// capped at 1,048,576 pixels before arithmetic. Ordinary asymmetric radii still
/// use proportional pair normalization, not a per-corner half-size clamp.
///
/// The resulting radii satisfy:
/// - all radii are non-negative
/// - top_left + top_right <= width
/// - bottom_left + bottom_right <= width
/// - top_left + bottom_left <= height
/// - top_right + bottom_right <= height
pub fn normalize_corner_radii(radii: CornerRadii, size: Vec2) -> CornerRadii {
    let width = bounded_extent(size.x);
    let height = bounded_extent(size.y);

    if width <= f32::EPSILON || height <= f32::EPSILON {
        return CornerRadii::ZERO;
    }

    let mut r = CornerRadii {
        top_left: bounded_extent(radii.top_left),
        top_right: bounded_extent(radii.top_right),
        bottom_right: bounded_extent(radii.bottom_right),
        bottom_left: bounded_extent(radii.bottom_left),
    };

    let top = r.top_left + r.top_right;
    let bottom = r.bottom_left + r.bottom_right;
    let left = r.top_left + r.bottom_left;
    let right = r.top_right + r.bottom_right;

    let scale_x = if top > 0.0 || bottom > 0.0 {
        let max_horizontal = top.max(bottom);
        if max_horizontal > 0.0 {
            (width / max_horizontal).min(1.0)
        } else {
            1.0
        }
    } else {
        1.0
    };

    let scale_y = if left > 0.0 || right > 0.0 {
        let max_vertical = left.max(right);
        if max_vertical > 0.0 {
            (height / max_vertical).min(1.0)
        } else {
            1.0
        }
    } else {
        1.0
    };

    let scale = scale_x.min(scale_y).clamp(0.0, 1.0);

    r.top_left *= scale;
    r.top_right *= scale;
    r.bottom_right *= scale;
    r.bottom_left *= scale;
    r
}

/// Normalizes border widths for inside-border semantics.
///
/// Widths are clamped to non-negative values and then proportionally scaled so
/// opposite sides never consume more than the available extent.
/// Inputs use the same finite extent bounds as `normalize_corner_radii`.
///
/// Guarantees:
/// - left + right <= width
/// - top + bottom <= height
pub fn normalize_border_widths(widths: BorderWidths, size: Vec2) -> BorderWidths {
    let width = bounded_extent(size.x);
    let height = bounded_extent(size.y);

    if width <= f32::EPSILON || height <= f32::EPSILON {
        return BorderWidths::ZERO;
    }

    let mut out = BorderWidths {
        top: bounded_extent(widths.top),
        right: bounded_extent(widths.right),
        bottom: bounded_extent(widths.bottom),
        left: bounded_extent(widths.left),
    };

    let horizontal_sum = out.left + out.right;
    if horizontal_sum > width && horizontal_sum > 0.0 {
        let scale = width / horizontal_sum;
        out.left *= scale;
        out.right *= scale;
    }

    let vertical_sum = out.top + out.bottom;
    if vertical_sum > height && vertical_sum > 0.0 {
        let scale = height / vertical_sum;
        out.top *= scale;
        out.bottom *= scale;
    }

    out
}

/// Signed distance to a rounded rectangle centered at the origin.
///
/// Coordinate convention:
/// - `local_pos = (0, 0)` is the rectangle center
/// - x grows right
/// - y grows down (to match UI UVs)
/// - `half_size` is half-width/half-height
///
/// Distance sign:
/// - d < 0 inside
/// - d = 0 boundary
/// - d > 0 outside
#[allow(dead_code)]
pub fn rounded_rect_signed_distance(local_pos: Vec2, half_size: Vec2, radii: CornerRadii) -> f32 {
    let radius = if local_pos.x >= 0.0 {
        if local_pos.y < 0.0 {
            radii.top_right
        } else {
            radii.bottom_right
        }
    } else if local_pos.y < 0.0 {
        radii.top_left
    } else {
        radii.bottom_left
    };

    let q = local_pos.abs() - half_size + Vec2::splat(radius);
    q.max(Vec2::ZERO).length() + q.x.max(q.y).min(0.0) - radius
}

#[cfg(test)]
mod tests {
    use super::{normalize_border_widths, normalize_corner_radii, rounded_rect_signed_distance};
    use crate::rendering::{BorderWidths, shape::CornerRadii};
    use bevy::prelude::*;

    fn nearly_le(a: f32, b: f32) -> bool {
        a <= b + 1e-4
    }

    #[test]
    fn normalize_keeps_valid_radii() {
        let radii = CornerRadii::corners(6.0, 10.0, 8.0, 4.0);
        let out = normalize_corner_radii(radii, Vec2::new(80.0, 40.0));
        assert_eq!(out, radii);
    }

    #[test]
    fn normalize_clamps_negative_radii() {
        let out = normalize_corner_radii(
            CornerRadii::corners(-6.0, 5.0, -10.0, 4.0),
            Vec2::new(50.0, 50.0),
        );
        assert_eq!(out.top_left, 0.0);
        assert_eq!(out.bottom_right, 0.0);
        assert_eq!(out.top_right, 5.0);
        assert_eq!(out.bottom_left, 4.0);
    }

    #[test]
    fn normalize_scales_when_horizontal_sums_exceed_width() {
        let out = normalize_corner_radii(
            CornerRadii::corners(80.0, 80.0, 10.0, 10.0),
            Vec2::new(100.0, 100.0),
        );
        assert!(nearly_le(out.top_left + out.top_right, 100.0));
        assert!(nearly_le(out.bottom_left + out.bottom_right, 100.0));
    }

    #[test]
    fn normalize_scales_when_vertical_sums_exceed_height() {
        let out = normalize_corner_radii(
            CornerRadii::corners(80.0, 20.0, 80.0, 20.0),
            Vec2::new(200.0, 100.0),
        );
        assert!(nearly_le(out.top_left + out.bottom_left, 100.0));
        assert!(nearly_le(out.top_right + out.bottom_right, 100.0));
    }

    #[test]
    fn normalize_returns_zero_for_empty_size() {
        let out = normalize_corner_radii(CornerRadii::new(12.0), Vec2::new(0.0, 20.0));
        assert_eq!(out, CornerRadii::ZERO);
    }

    #[test]
    fn sdf_sign_is_consistent() {
        let radii = CornerRadii::new(8.0);
        let half = Vec2::new(20.0, 12.0);

        let inside = rounded_rect_signed_distance(Vec2::new(0.0, 0.0), half, radii);
        let boundary = rounded_rect_signed_distance(Vec2::new(20.0, 0.0), half, radii);
        let outside = rounded_rect_signed_distance(Vec2::new(26.0, 0.0), half, radii);

        assert!(inside < 0.0);
        assert!(boundary.abs() < 1e-4);
        assert!(outside > 0.0);
    }

    #[test]
    fn normalize_border_widths_clamps_negative() {
        let widths = BorderWidths::sides(-1.0, 3.0, -2.0, 4.0);
        let out = normalize_border_widths(widths, Vec2::new(100.0, 50.0));
        assert_eq!(out.top, 0.0);
        assert_eq!(out.right, 3.0);
        assert_eq!(out.bottom, 0.0);
        assert_eq!(out.left, 4.0);
    }

    #[test]
    fn normalize_border_widths_scales_horizontal_pair() {
        let widths = BorderWidths::sides(2.0, 80.0, 2.0, 80.0);
        let out = normalize_border_widths(widths, Vec2::new(100.0, 100.0));
        assert!(nearly_le(out.left + out.right, 100.0));
    }

    #[test]
    fn normalize_border_widths_scales_vertical_pair() {
        let widths = BorderWidths::sides(80.0, 2.0, 80.0, 2.0);
        let out = normalize_border_widths(widths, Vec2::new(100.0, 100.0));
        assert!(nearly_le(out.top + out.bottom, 100.0));
    }

    #[test]
    fn normalize_border_widths_empty_size_returns_zero() {
        let out = normalize_border_widths(BorderWidths::all(8.0), Vec2::new(0.0, 100.0));
        assert_eq!(out, BorderWidths::ZERO);
    }

    #[test]
    fn normalizers_bound_non_finite_and_extreme_inputs_before_arithmetic() {
        let values = [
            f32::NAN, f32::INFINITY, f32::NEG_INFINITY, f32::MAX, f32::MIN,
            0.0, f32::from_bits(1), 1.0, 100.0, super::MAX_GEOMETRY_EXTENT,
        ];
        for width in values {
            for height in values {
                for value in values {
                    let size = Vec2::new(width, height);
                    let radii = normalize_corner_radii(
                        CornerRadii::corners(value, 8.0, value, 0.0), size,
                    );
                    let borders = normalize_border_widths(
                        BorderWidths::sides(value, 8.0, value, 0.0), size,
                    );
                    for out in [radii.as_vec4(), borders.as_vec4()] {
                        assert!(out.is_finite(), "size={size:?}, value={value}, out={out:?}");
                        assert!(out.cmpge(Vec4::ZERO).all());
                        assert!(out.cmple(Vec4::splat(super::MAX_GEOMETRY_EXTENT)).all());
                    }
                    let w = super::bounded_extent(width);
                    let h = super::bounded_extent(height);
                    // Relative tolerance accounts for f32 pair-sum rounding at
                    // the upper bound, not just tiny widget-sized inputs.
                    let le = |sum: f32, limit: f32| sum <= limit + limit * 1e-6 + 1e-4;
                    assert!(le(radii.top_left + radii.top_right, w));
                    assert!(le(radii.bottom_left + radii.bottom_right, w));
                    assert!(le(radii.top_left + radii.bottom_left, h));
                    assert!(le(radii.top_right + radii.bottom_right, h));
                    assert!(le(borders.left + borders.right, w));
                    assert!(le(borders.top + borders.bottom, h));
                }
            }
        }
    }

    #[test]
    fn huge_positive_pairs_normalize_instead_of_collapsing_to_zero_or_nan() {
        for value in [f32::INFINITY, f32::MAX] {
            let size = Vec2::new(100.0, 100.0);
            assert_eq!(
                normalize_corner_radii(CornerRadii::new(value), size),
                CornerRadii::new(50.0),
            );
            assert_eq!(
                normalize_border_widths(BorderWidths::all(value), size),
                BorderWidths::all(50.0),
            );
        }
    }

    #[test]
    fn normalizers_preserve_valid_asymmetry_and_large_extents() {
        let radii = CornerRadii::corners(80.0, 10.0, 5.0, 10.0);
        let borders = BorderWidths::sides(80.0, 10.0, 5.0, 70.0);
        let size = Vec2::splat(100.0);
        assert_eq!(normalize_corner_radii(radii, size), radii);
        assert_eq!(normalize_border_widths(borders, size), borders);

        let radii = CornerRadii::new(16_384.0);
        let borders = BorderWidths::all(16_384.0);
        let size = Vec2::splat(65_536.0);
        assert_eq!(normalize_corner_radii(radii, size), radii);
        assert_eq!(normalize_border_widths(borders, size), borders);
    }
}
