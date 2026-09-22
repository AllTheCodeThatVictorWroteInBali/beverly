//! Focused shader regressions. Numerical tests mirror the coverage algebra;
//! source assertions tie those formulas to the production shader. They are not
//! GPU readback tests. The debug-build shader-cache test additionally runs the
//! real WGSL through Bevy's validating naga-oil composer, with minimal imports.

use bevy::prelude::*;

const SHADER: &str = include_str!("shaders/ui_shape.wgsl");

fn premul(color: Vec4) -> Vec4 {
    (color.truncate() * color.w).extend(color.w)
}

fn over(bottom: Vec4, top: Vec4) -> Vec4 {
    top + bottom * (1.0 - top.w)
}

fn paint_regions(fill: Vec4, border: Vec4, outer: f32, inner: f32, mask: f32) -> Vec4 {
    let border_coverage = (outer - inner).clamp(0.0, 1.0);
    (premul(fill) * inner + premul(border) * border_coverage) * mask
}

fn assert_color(actual: Vec4, expected: Vec4) {
    assert!(actual.abs_diff_eq(expected, 1e-6), "{actual:?} != {expected:?}");
}

#[test]
fn shader_sums_disjoint_paint_after_region_noise_before_source_over() {
    assert!(SHADER.contains("let border_coverage = clamp(outer_coverage - fill_coverage, 0.0, 1.0);"));
    assert!(SHADER.contains("return top + bottom * (1.0 - top.a);"));
    assert!(SHADER.contains("let paint_layer = masked_border + masked_fill;"));
    assert!(SHADER.contains("surface_layer = composite_over(surface_layer, paint_layer);"));
    assert!(!SHADER.contains("composite_over(surface_layer, masked_fill)"));
    assert!(!SHADER.contains("composite_over(surface_layer, masked_border)"));
    let sum = SHADER.find("let paint_layer =").unwrap();
    for region in ["masked_fill", "masked_border"] {
        assert!(SHADER.find(&format!("{region} = modulate_luminance_premul")).unwrap() < sum);
    }
    assert!(sum < SHADER.find("surface_layer = modulate_luminance_premul").unwrap());
}

#[test]
fn opaque_half_covered_fill_and_border_have_no_alpha_seam_or_color_bias() {
    let red = Vec4::new(1.0, 0.0, 0.0, 1.0);
    let blue = Vec4::new(0.0, 0.0, 1.0, 1.0);
    let paint = paint_regions(red, blue, 1.0, 0.5, 1.0);
    assert_color(paint, Vec4::new(0.5, 0.0, 0.5, 1.0));
    assert_color(over(Vec4::new(0.0, 1.0, 0.0, 1.0), paint), paint);
    // Explicitly distinguish the old source-over result (alpha 0.75).
    assert_color(over(blue * 0.5, red * 0.5), Vec4::new(0.5, 0.0, 0.25, 0.75));
}

#[test]
fn translucent_regions_sum_premultiplied_colors_then_composite_once() {
    let red = Vec4::new(1.0, 0.0, 0.0, 0.4);
    let blue = Vec4::new(0.0, 0.0, 1.0, 0.6);
    let green = Vec4::new(0.0, 1.0, 0.0, 1.0);
    let paint = paint_regions(red, blue, 1.0, 0.5, 1.0);
    assert_color(paint, Vec4::new(0.2, 0.0, 0.3, 0.5));
    assert_color(over(green, paint), Vec4::new(0.2, 0.5, 0.3, 1.0));
    let masked = paint_regions(red, blue, 1.0, 0.5, 0.4);
    assert_color(over(green, masked), Vec4::new(0.08, 0.8, 0.12, 1.0));
}

#[test]
fn backdrop_includes_shape_coverage_but_paint_does_not_square_edge_aa() {
    assert!(SHADER.contains("let surface_mask = clamp(clip_mask * mask_coverage, 0.0, 1.0);"));
    assert!(SHADER.contains("let effective_surface_coverage = outer_coverage * surface_mask;"));
    assert!(SHADER.contains("backdrop_processed.rgb * effective_surface_coverage"));
    assert!(SHADER.contains("backdrop_processed.a * effective_surface_coverage"));
    assert!(SHADER.contains("var masked_fill = fill_layer * surface_mask;"));
    assert!(SHADER.contains("var masked_border = border_layer * surface_mask;"));
    assert!(SHADER.contains("let masked_inner_shadow = inner_shadow_layer * surface_mask;"));

    for (outer, fill) in [(0.0, 0.0), (0.5, 0.5), (0.6, 0.2), (1.0, 0.5)] {
        for (clip, mask) in [(1.0_f32, 1.0_f32), (0.5, 0.4), (0.0, 1.0)] {
            let surface_mask = (clip * mask).clamp(0.0, 1.0);
            let backdrop = Vec4::ONE * (outer * surface_mask);
            let paint = paint_regions(Vec4::ONE, Vec4::ONE, outer, fill, surface_mask);
            assert_color(paint, backdrop);
            assert!((paint.w - outer * surface_mask).abs() < 1e-6);
        }
    }
}

#[test]
fn focus_debug_modes_are_reached_before_noise_modulation() {
    // Inspect actual shader branch order, not a second Rust dispatch table.
    let thresholds: Vec<f32> = SHADER.lines().filter_map(|line| {
        line.trim().strip_prefix("if render_debug_mode > ")?
            .strip_suffix(" {")?.parse().ok()
    }).collect();
    assert_eq!(thresholds, vec![9.5, 8.5, 7.5, 6.5, 5.5, 4.5, 3.5, 2.5, 1.5, 0.5]);
    for (mode, expected) in [(8.0, 7.5), (9.0, 8.5), (10.0, 9.5)] {
        assert_eq!(thresholds.iter().find(|&&threshold| mode > threshold), Some(&expected));
    }
}

#[test]
fn css_asymmetry_requires_more_than_removing_the_quadrant_sdf_clamp() {
    use crate::rendering::{sdf::{normalize_corner_radii, rounded_rect_signed_distance}, shape::CornerRadii};

    let half = Vec2::splat(50.0);
    let radii = normalize_corner_radii(CornerRadii::corners(80.0, 0.0, 0.0, 0.0), half * 2.0);
    assert_eq!(radii.top_left, 80.0); // Valid CSS geometry, larger than half-size.
    let left = Vec2::new(-0.001, -49.0);
    let right = Vec2::new(0.001, -49.0);
    let left_distance = rounded_rect_signed_distance(left, half, radii);
    let right_distance = rounded_rect_signed_distance(right, half, radii);
    // The true TL quarter-circle still occupies BOTH points' quadrants.
    let actual_right_distance = (right - Vec2::splat(30.0)).length() - 80.0;
    assert!(left_distance > 4.0 && actual_right_distance > 4.0);
    assert!(right_distance < 0.0, "quadrant SDF incorrectly reports inside");

    let clamped = CornerRadii::corners(50.0, 0.0, 0.0, 0.0);
    assert!((rounded_rect_signed_distance(left, half, clamped)
        - rounded_rect_signed_distance(right, half, clamped)).abs() < 1e-4);
    assert!(SHADER.contains("let r = corner_radius(local, radii);"));
    assert!(SHADER.contains("let radii = clamp(uniforms.corner_radii, vec4<f32>(0.0), vec4<f32>(max_radius));"));
}

#[test]
fn glass_uniform_contract_and_transformed_expansion_match_parent_abi() {
    let abi = SHADER.split("struct UiShapeUniforms {").nth(1).unwrap()
        .split("};").next().unwrap();
    let fields: Vec<_> = abi.lines().map(str::trim)
        .filter(|line| !line.starts_with("//") && line.contains(':')).collect();
    let start = fields.iter().position(|line| line.starts_with("backdrop_uv_rect:")).unwrap();
    assert_eq!(&fields[start..], &[
        "backdrop_uv_rect: vec4<f32>,",
        "glass_optics: vec4<f32>,",
        "glass_light: vec4<f32>,",
        "glass_state: vec4<f32>,",
        "glass_debug: vec4<f32>,",
        "surface_axes: vec4<f32>,",
        "debug_view: f32,",
    ]);
    let vertex = SHADER.split("fn vertex(").nth(1).unwrap().split("fn stop_position").next().unwrap();
    assert!(vertex.contains("uniforms.surface_axes.xy * expand.x + uniforms.surface_axes.zw * expand.y"));
    assert!(!vertex.contains("glass_state"), "press must not scale content geometry");
    assert!(vertex.contains("out.uv = vertex_uv;"));
}

#[test]
fn glass_reuses_canonical_sdf_and_one_explicit_lod_blur_kernel() {
    assert_eq!(SHADER.matches("fn sdf_rounded_rect(").count(), 1);
    let normals = SHADER.split("fn glass_sdf_normal(").nth(1).unwrap()
        .split("fn glass_screen_slope(").next().unwrap();
    assert_eq!(normals.matches("sdf_rounded_rect(").count(), 4);
    assert!(!SHADER.contains("textureSample("));
    assert_eq!(SHADER.matches("textureSampleLevel(").count(), 1);
    assert!(SHADER.contains("textureSampleLevel(backdrop_texture, backdrop_sampler, uv, 0.0)"));
    assert!(!SHADER.contains("uniforms.backdrop_uv_rect"), "AABB must not map capture UVs");
    assert!(SHADER.contains("let screen_uv = in.position.xy / capture_size;"));
    let glass = SHADER.split("fn sample_glass_backdrop(").nth(1).unwrap()
        .split("fn glass_debug_color(").next().unwrap();
    assert_eq!(glass.matches("sample_backdrop_blurred(").count(), 1);
    assert_eq!(glass.matches("sample_backdrop_source(").count(), 2);
    assert!(glass.contains("center_blur * (2.5 / 1.4)"));
    assert!(glass.contains("(red.rgb / max(red.a, 1e-5) - blue.rgb / max(blue.a, 1e-5)) * sample.a"));
    assert!(glass.contains("sample.rgb + vec3<f32>(delta.r, 0.0, -delta.b)"));
}

#[test]
fn glass_sampling_is_guarded_and_debug_preserves_capture_alpha() {
    let fragment = SHADER.split("fn fragment(").nth(1).unwrap();
    let guard = fragment.find("if backdrop_enabled {").unwrap();
    let end = fragment.find("var masked_border =").unwrap();
    let guarded = &fragment[guard..end];
    assert!(!fragment[..guard].contains("sample_backdrop_"));
    assert!(!fragment[end..].contains("sample_backdrop_"));
    assert!(guarded.contains("} else if backdrop_debug_mode > 0.5 || uniforms.glass_debug.x > 0.5 {\n        return vec4<f32>(0.0);"));
    assert!(!guarded.contains("to_premul("));
    assert_eq!(guarded.matches("return from_premul(dbg * effective_surface_coverage);").count(), 3);
    assert!(guarded.find("if backdrop_debug_mode > 1.5").unwrap()
        < guarded.find("evaluate_glass_optics(").unwrap());
    assert!(guarded.find("process_backdrop_color(optical_sample)").unwrap()
        < guarded.find("let reflection =").unwrap());
    assert!(fragment.find("composite_over(surface_layer, backdrop_layer)").unwrap()
        < fragment.find("composite_over(surface_layer, paint_layer)").unwrap());
    assert!(fragment.find("composite_over(out_premul, surface_layer)").unwrap()
        < fragment.find("composite_over(out_premul, focus_primary_layer)").unwrap());
    assert!(fragment.contains("return from_premul(vec4<f32>(clamped_rgb, clamped_alpha));"));

    // Capture already carries alpha. Coverage scales it once; the native
    // ALPHA_BLENDING pipeline receives straight RGB (including in diagnostics).
    let capture = premul(Vec4::new(0.8, 0.4, 0.2, 0.5));
    let masked = capture * 0.3;
    let straight = (masked.truncate() / masked.w).extend(masked.w);
    assert_color(straight, Vec4::new(0.8, 0.4, 0.2, 0.15));
    let reflected = masked.truncate().lerp(Vec3::splat(masked.w), 0.28).extend(masked.w);
    assert_eq!(reflected.w, masked.w);
    assert!(reflected.truncate().cmple(Vec3::splat(reflected.w)).all());
}

// Numerical mirrors test the bounds, not GPU execution. Source contracts and
// the real shader's naga tests below keep these tied to the production path.
fn glass_height(x: f32, profile: u32) -> f32 {
    let q = 1.0 - x.clamp(0.0, 1.0);
    let convex = (1.0 - q * q).max(0.0).sqrt();
    match profile {
        0 => convex,
        1 => (1.0 - q.powi(4)).max(0.0).sqrt().sqrt(),
        2 => 1.0 - convex,
        _ => {
            let t = ((x - 0.15) / 0.7).clamp(0.0, 1.0);
            let blend = t * t * (3.0 - 2.0 * t);
            convex * (1.0 - blend) + (1.0 - convex) * blend
        }
    }
}

fn glass_derivative(x: f32, profile: u32) -> f32 {
    let lo = (x - 0.01).max(0.0);
    let hi = (x + 0.01).min(1.0);
    let derivative = (glass_height(hi, profile) - glass_height(lo, profile)) / (hi - lo).max(0.01);
    let t = ((1.0 - x) / 0.15).clamp(0.0, 1.0);
    derivative.clamp(-4.0, 4.0) * t * t * (3.0 - 2.0 * t)
}

#[test]
fn glass_profiles_and_snell_displacement_are_bounded_with_calm_centers() {
    assert!(SHADER.contains("clamp(derivative, -4.0, 4.0) * smoothstep(0.0, 0.15, 1.0 - x)"));
    assert!(SHADER.contains("refract(vec3<f32>(0.0, 0.0, -1.0), local_normal, 1.0 / ior)"));
    assert!(SHADER.contains("let limit = bezel * 0.35;"));
    for profile in 0..4 {
        assert_eq!(glass_derivative(1.0, profile), 0.0);
        for step in 0..=1000 {
            let x = step as f32 / 1000.0;
            let slope = glass_derivative(x, profile);
            assert!(slope.is_finite() && slope.abs() <= 4.0 + 1e-5);
            for press in [0.0, 0.5, 1.0] {
                let normal = Vec3::new(slope * (3.0 / 4.5) * (1.0 + 0.12 * press), 0.0, 1.0).normalize();
                for ior in [1.0, 1.3, 1.8] {
                    let eta = 1.0 / ior;
                    let incident = -Vec3::Z;
                    let dot = normal.dot(incident);
                    let k = 1.0 - eta * eta * (1.0 - dot * dot);
                    assert!(k >= 0.0, "air-to-glass has no total internal reflection");
                    let ray = eta * incident - (eta * dot + k.sqrt()) * normal;
                    let mut displacement = ray.truncate() * (3.0 / (-ray.z).max(0.1));
                    displacement *= (4.5 * 0.35 / displacement.length().max(1e-4)).min(1.0);
                    assert!(displacement.is_finite());
                    assert!(displacement.length() <= 4.5 * 0.35 + 1e-5);
                    if x == 1.0 || ior == 1.0 {
                        assert!(displacement.length() < 1e-5);
                    }
                }
            }
        }
    }
    assert!(glass_derivative(0.1, 0) > 0.0);
    assert!(glass_derivative(0.1, 2) < 0.0);
    assert!(glass_derivative(0.1, 3) > 0.0 && glass_derivative(0.5, 3) < 0.0);
}

fn screen_slope(axes: Mat2, slope: Vec2) -> Vec2 {
    let scale = axes.x_axis.length().max(axes.y_axis.length());
    let a = axes.x_axis / scale.max(1e-4);
    let b = axes.y_axis / scale.max(1e-4);
    let det = a.x * b.y - b.x * a.y;
    if scale < 1e-4 || det.abs() < 1e-4 {
        return Vec2::ZERO;
    }
    Vec2::new(b.y * slope.x - a.y * slope.y, -b.x * slope.x + a.x * slope.y) / (det * scale)
}

#[test]
fn glass_normals_use_inverse_transpose_and_singular_axes_stay_finite() {
    assert!(SHADER.contains("if scale < 1e-4 || abs(det) < 1e-4"));
    assert!(SHADER.contains("b.y * slope.x - a.y * slope.y, -b.x * slope.x + a.x * slope.y"));
    assert!(SHADER.contains("uniforms.surface_axes.xy * displacement.x + uniforms.surface_axes.zw * displacement.y"));
    let slope = Vec2::new(0.3, -0.7);
    for axes in [
        Mat2::IDENTITY,
        Mat2::from_angle(0.8),
        Mat2::from_angle(0.8) * Mat2::from_diagonal(Vec2::new(2.0, 0.4)),
        Mat2::from_cols(Vec2::new(-2.0, 0.3), Vec2::new(0.8, 1.5)),
    ] {
        let transformed = screen_slope(axes, slope);
        assert!(transformed.abs_diff_eq(axes.inverse().transpose() * slope, 1e-5));
        let tangent = Vec2::new(0.7, 0.3);
        assert!((transformed.dot(axes * tangent) - slope.dot(tangent)).abs() < 1e-5);
    }
    for axes in [Mat2::ZERO, Mat2::from_cols(Vec2::X, Vec2::X), Mat2::from_diagonal(Vec2::new(1.0, 1e-8))] {
        assert_eq!(screen_slope(axes, slope), Vec2::ZERO);
    }
}

// Numerical shimmer mirrors consume the real Rust encoder's packed descriptor.
// Source assertions below bind the algebra to WGSL; none execute on a GPU.
fn shimmer_progress(time: f32, paint: &super::PaintUniform) -> f32 {
    (time / paint.linear_points.x + paint.linear_points.z).fract()
}

fn shimmer_center(progress: f32, paint: &super::PaintUniform) -> f32 {
    let half_width = paint.linear_points.w * 0.5;
    -half_width * (1.0 - progress) + (1.0 + half_width) * progress
}

fn shimmer_strength(x: f32, progress: f32, paint: &super::PaintUniform) -> f32 {
    let half_width = paint.linear_points.w * 0.5;
    let x = if paint.linear_points.y < 0.0 { 1.0 - x } else { x };
    let distance = (x - shimmer_center(progress, paint)).abs();
    let edge_start = half_width * (1.0 - paint.radial_center_radius.x);
    let t = ((distance - edge_start) / (half_width - edge_start)).clamp(0.0, 1.0);
    (1.0 - t * t * (3.0 - 2.0 * t)) * paint.radial_center_radius.y
}

fn shimmer_color(x: f32, time: f32, paint: &super::PaintUniform) -> Vec4 {
    if paint.kind_and_flags.w < 0.5 {
        return paint.solid_color;
    }
    let strength = shimmer_strength(x, shimmer_progress(time, paint), paint);
    let mixed = premul(paint.solid_color).lerp(premul(paint.angular_center_angle), strength);
    if mixed.w <= 1e-5 { Vec4::ZERO } else { (mixed.truncate() / mixed.w).extend(mixed.w) }
}

#[test]
fn shimmer_wgsl_source_binds_packed_kind4_timing_bounds_and_premul_math() {
    let body = SHADER.split("fn evaluate_shimmer(").nth(1).unwrap()
        .split("fn evaluate_paint(").next().unwrap();
    for statement in [
        "if paint.kind_and_flags.w < 0.5 {\n        return paint.solid_color;",
        "let progress = fract(globals.time / paint.linear_points.x + paint.linear_points.z);",
        "let half_width = paint.linear_points.w * 0.5;",
        "let center = mix(-half_width, 1.0 + half_width, progress);",
        "let x = select(uv.x, 1.0 - uv.x, paint.linear_points.y < 0.0);",
        "let distance = abs(x - center);",
        "let edge_start = half_width * (1.0 - paint.radial_center_radius.x);",
        "let strength = (1.0 - smoothstep(edge_start, half_width, distance)) * paint.radial_center_radius.y;",
        "return from_premul(mix(to_premul(paint.solid_color), to_premul(paint.angular_center_angle), strength));",
    ] {
        assert!(body.contains(statement), "missing shimmer contract: {statement}");
    }
    assert!(body.find("return paint.solid_color;").unwrap() < body.find("globals.time").unwrap());
    for forbidden in ["uv.y", "sin(", "textureSample", "stops[", "corner_radii"] {
        assert!(!body.contains(forbidden), "shimmer must remain a local horizontal paint: {forbidden}");
    }
    let dispatch = SHADER.split("fn evaluate_paint(").nth(1).unwrap()
        .split("fn dither_noise(").next().unwrap();
    assert!(dispatch.contains("if kind > 3.5 {\n        return evaluate_shimmer(uv, paint);"));
    assert!(dispatch.find("return evaluate_shimmer").unwrap() < dispatch.find("if kind < 0.5").unwrap());
    assert!(SHADER.contains("return vec4<f32>(color.rgb * color.a, color.a);"));
    assert!(SHADER.contains("if color.a <= 1e-5 {\n        return vec4<f32>(0.0, 0.0, 0.0, 0.0);"));
    assert!(SHADER.contains("return vec4<f32>(color.rgb / color.a, color.a);"));
    assert!(SHADER.contains("let fill_color = evaluate_paint(surface_uv, uniforms.fill_paint);"));
    assert!(SHADER.contains("let fill_layer = to_premul(fill_color) * fill_coverage;"));
}

#[test]
fn shimmer_sweep_seam_is_offscreen_for_both_directions_and_band_limits() {
    use crate::rendering::{Paint, Shimmer, ShimmerDirection};
    for direction in [ShimmerDirection::LeftToRight, ShimmerDirection::RightToLeft] {
        for width in [0.01, 0.22, 0.8] {
            for softness in [0.05, 0.8, 1.0] {
                let paint = super::encode_paint(&Paint::Shimmer(Shimmer {
                    direction, width, softness, intensity: 1.0, ..default()
                }));
                let half_width = width * 0.5;
                assert!(shimmer_center(0.0, &paint) + half_width <= 0.0);
                assert!(shimmer_center(1.0, &paint) - half_width >= 1.0 - 1e-7);
                for step in 0..=100 {
                    let x = step as f32 / 100.0;
                    for progress in [0.0, 1.0] {
                        assert!(shimmer_strength(x, progress, &paint) < 1e-6);
                    }
                    // Approach each side of fract's discontinuity, not just the
                    // exact endpoints: even the narrowest/hardest band fades out.
                    for progress in [1e-6, 1.0 - 1e-6] {
                        assert!(shimmer_strength(x, progress, &paint) < 0.001);
                    }
                }
                assert!((shimmer_strength(0.5, 0.5, &paint) - 1.0).abs() < 1e-6);
            }
        }
    }
}

#[test]
fn shimmer_width_and_softness_define_plateau_feather_and_zero_outside() {
    use crate::rendering::{Paint, Shimmer};
    // Binary-exact inputs let the feather midpoint distinguish smoothstep from
    // a hard edge or a misplaced full-width/half-width boundary without drift.
    for width in [0.125, 0.25, 0.5] {
        for softness in [0.125, 0.5, 1.0] {
            let paint = super::encode_paint(&Paint::Shimmer(Shimmer {
                width, softness, intensity: 0.75, ..default()
            }));
            let half_width = width * 0.5;
            let edge_start = half_width * (1.0 - softness);
            for sign in [-1.0, 1.0] {
                assert_eq!(shimmer_strength(0.5 + sign * edge_start, 0.5, &paint), 0.75);
                assert_eq!(shimmer_strength(0.5 + sign * (edge_start + half_width) * 0.5,
                    0.5, &paint), 0.375);
                assert_eq!(shimmer_strength(0.5 + sign * half_width, 0.5, &paint), 0.0);
                assert_eq!(shimmer_strength(0.5 + sign * width, 0.5, &paint), 0.0);
                let mut previous = 0.75;
                for step in 0..=32 {
                    let x = 0.5 + sign * half_width * step as f32 / 32.0;
                    let strength = shimmer_strength(x, 0.5, &paint);
                    assert!(strength <= previous);
                    previous = strength;
                }
            }
        }
    }
}

#[test]
fn shimmer_ltr_rtl_are_spatial_mirrors_and_move_in_opposite_directions() {
    use crate::rendering::{Paint, Shimmer, ShimmerDirection};
    let ltr = super::encode_paint(&Paint::Shimmer(Shimmer { intensity: 1.0, ..default() }));
    let rtl = super::encode_paint(&Paint::Shimmer(Shimmer {
        direction: ShimmerDirection::RightToLeft, intensity: 1.0, ..default()
    }));
    for step in 0..=64 {
        let progress = step as f32 / 64.0;
        for position in 0..=64 {
            let x = position as f32 / 64.0;
            assert!((shimmer_strength(x, progress, &ltr)
                - shimmer_strength(1.0 - x, progress, &rtl)).abs() < 1e-6);
        }
    }
    let first = shimmer_center(0.25, &ltr);
    let last = shimmer_center(0.75, &ltr);
    assert!(first > 0.0 && first < last && last < 1.0);
    assert!(1.0 - first > 1.0 - last);
    for progress in [0.25, 0.75] {
        let center = shimmer_center(progress, &ltr);
        assert_eq!(shimmer_strength(center, progress, &ltr), 1.0);
        assert_eq!(shimmer_strength(1.0 - center, progress, &rtl), 1.0);
        assert_eq!(shimmer_strength(center, progress, &rtl), 0.0);
    }
}

#[test]
fn shimmer_duration_and_phase_use_shared_seconds_not_mount_time() {
    use crate::rendering::{Paint, Shimmer};
    for duration in [0.1, 1.5, 2.0, 3600.0] {
        for (phase, normalized) in [(-0.25, 0.75), (0.0, 0.0), (0.25, 0.25), (3.25, 0.25)] {
            let paint = super::encode_paint(&Paint::Shimmer(Shimmer { duration, phase, ..default() }));
            for fraction in [0.0, 0.125, 0.5, 0.875] {
                let time = duration * fraction;
                let expected = (fraction + normalized).fract();
                assert!((shimmer_progress(time, &paint) - expected).abs() < 1e-6);
                assert!((shimmer_progress(time + duration, &paint) - expected).abs() < 1e-6);
            }
        }
    }
}

#[test]
fn shimmer_bevy_globals_rollover_is_not_an_arbitrary_duration_cycle_seam() {
    use crate::rendering::{Paint, Shimmer};
    use std::time::Duration;
    let mut time = Time::<()>::default();
    assert_eq!(time.wrap_period(), Duration::from_secs(3600));
    time.advance_by(Duration::from_millis(3_599_750));
    assert_eq!(time.elapsed_secs_wrapped(), 3599.75);
    time.advance_by(Duration::from_millis(250));
    assert_eq!(time.elapsed_secs_wrapped(), 0.0);
    assert_eq!(time.elapsed_secs(), 3600.0);
    let paint = super::encode_paint(&Paint::Shimmer(Shimmer { duration: 7.0, ..default() }));
    assert_eq!(shimmer_progress(time.elapsed_secs_wrapped(), &paint), 0.0);
    assert!(shimmer_progress(time.elapsed_secs(), &paint) > 0.28);
    // The default 1.5s duration divides the default wrap period; seven seconds
    // does not. This is a clock reset, not a guarantee of seamless long uptime.
    let default_paint = super::encode_paint(&Paint::Shimmer(Shimmer::default()));
    assert_eq!(shimmer_progress(time.elapsed_secs(), &default_paint), 0.0);
    time.set_wrap_period(Duration::from_secs(8));
    time.advance_by(Duration::from_secs(1));
    assert_eq!(time.elapsed_secs_wrapped(), 1.0);
}

#[test]
fn shimmer_invalid_extreme_width_softness_and_time_samples_stay_finite_and_bounded() {
    use crate::rendering::{Paint, Shimmer};
    let values = [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, f32::MIN,
        -0.0, f32::from_bits(1), 0.5, f32::MAX];
    for width in values {
        for softness in values {
            let paint = super::encode_paint(&Paint::Shimmer(Shimmer {
                width, softness, duration: width, phase: softness,
                intensity: softness, ..default()
            }));
            let edge_span = paint.linear_points.w * 0.5 * paint.radial_center_radius.x;
            assert!(edge_span.is_finite() && edge_span > 0.0);
            // Valid GPU globals range, not arbitrary NaN/overflowing clock input.
            for time in [0.0, 0.05, 1.5, 1800.0, 3599.9998] {
                let progress = shimmer_progress(time, &paint);
                assert!(progress.is_finite() && (0.0..1.0).contains(&progress));
                for x in [-f32::MAX, -1.0, 0.0, 0.25, 0.5, 0.75, 1.0, 2.0, f32::MAX] {
                    let strength = shimmer_strength(x, progress, &paint);
                    assert!(strength.is_finite() && (0.0..=paint.radial_center_radius.y).contains(&strength));
                }
            }
        }
    }
}

#[test]
fn shimmer_translucent_interpolation_is_premultiplied_and_disabled_is_exact_base() {
    use crate::rendering::{Paint, Shimmer};
    let shimmer = Shimmer {
        base_color: Color::linear_rgba(1.0, 0.0, 0.0, 0.25),
        highlight_color: Color::linear_rgba(0.0, 0.0, 1.0, 0.75),
        intensity: 0.5, ..default()
    };
    let paint = super::encode_paint(&Paint::Shimmer(shimmer));
    let mixed = shimmer_color(0.5, shimmer.duration * 0.5, &paint);
    assert_color(mixed, Vec4::new(0.25, 0.0, 0.75, 0.5));
    assert_color(premul(mixed), Vec4::new(0.125, 0.0, 0.375, 0.5));
    assert!(!mixed.abs_diff_eq(paint.solid_color.lerp(paint.angular_center_angle, 0.5), 1e-3));
    let transparent_base = super::encode_paint(&Paint::Shimmer(Shimmer {
        base_color: Color::linear_rgba(1.0, 0.0, 0.0, 0.0), ..shimmer
    }));
    assert_color(shimmer_color(0.5, 0.75, &transparent_base), Vec4::new(0.0, 0.0, 1.0, 0.375));
    let transparent_highlight = super::encode_paint(&Paint::Shimmer(Shimmer {
        highlight_color: Color::linear_rgba(0.0, 0.0, 1.0, 0.0), ..shimmer
    }));
    assert_color(shimmer_color(0.5, 0.75, &transparent_highlight), Vec4::new(1.0, 0.0, 0.0, 0.125));
    let transparent_both = super::encode_paint(&Paint::Shimmer(Shimmer {
        base_color: Color::linear_rgba(1.0, 0.0, 0.0, 0.0),
        highlight_color: Color::linear_rgba(0.0, 0.0, 1.0, 0.0), ..shimmer
    }));
    assert_eq!(shimmer_color(0.5, 0.75, &transparent_both), Vec4::ZERO);
    for intensity in [0.0, 1.0] {
        let paint = super::encode_paint(&Paint::Shimmer(Shimmer { intensity, ..shimmer }));
        assert_color(shimmer_color(0.5, 0.75, &paint),
            if intensity == 0.0 { paint.solid_color } else { paint.angular_center_angle });
    }
    let disabled = super::encode_paint(&Paint::Shimmer(Shimmer { enabled: false, ..shimmer }));
    for time in [0.0, 0.75, 3599.0, f32::NAN] {
        assert_eq!(shimmer_color(0.5, time, &disabled), disabled.solid_color);
    }
}

// ShaderCache::new enables naga-oil validation in debug builds only. Do not
// claim validation from a release build's non-validating composer.
#[cfg(debug_assertions)]
fn validate_wgsl(source: &str) -> Result<(), String> {
    use bevy::{
        render::render_resource::{DownlevelFlags, WgpuFeatures},
        shader::{Shader, ShaderCache},
    };

    // Minimal stand-ins for external Bevy imports: validate the entire real
    // shader body and both entry points, without a GPU, window, or asset server.
    // Import ABI/pipeline compatibility still needs renderer runtime validation.
    let imports = [
        "#define_import_path bevy_render::view\nstruct View { clip_from_world: mat4x4<f32>, };",
        "#define_import_path bevy_render::globals\nstruct Globals { time: f32, };",
        "#define_import_path bevy_ui::ui_vertex_output\nstruct UiVertexOutput {\n\
            @builtin(position) position: vec4<f32>,\n\
            @location(0) uv: vec2<f32>,\n\
            @location(1) size: vec2<f32>,\n\
            @location(2) border_widths: vec4<f32>,\n\
            @location(3) border_radius: vec4<f32>,\n};",
    ];
    let mut assets = Assets::<Shader>::default();
    let mut cache = ShaderCache::new((), WgpuFeatures::empty(), DownlevelFlags::empty(), |_, _, _| Ok(()));
    for (index, import) in imports.iter().enumerate() {
        let shader = Shader::from_wgsl(*import, format!("test_import_{index}.wgsl"));
        let handle = assets.add(shader.clone());
        cache.set_shader(handle.id(), shader);
    }
    let shader = Shader::from_wgsl(source.to_owned(), "ui_shape.wgsl");
    let handle = assets.add(shader.clone());
    cache.set_shader(handle.id(), shader);
    cache.get(0, handle.id(), &[]).map(|_| ()).map_err(|error| format!("{error:?}"))
}

#[cfg(debug_assertions)]
#[test]
fn real_ui_shape_wgsl_passes_bevy_naga_validation() {
    validate_wgsl(SHADER).expect("production UI shape shader must validate");
    // Negative control: syntactically valid WGSL with incompatible operands
    // must fail semantic validation rather than silently accepting the source.
    let invalid = SHADER.replace(
        "let paint_layer = masked_border + masked_fill;",
        "let paint_layer = masked_border + masked_fill.xyz;",
    );
    assert_ne!(invalid, SHADER);
    assert!(validate_wgsl(&invalid).is_err());
}

#[cfg(debug_assertions)]
#[test]
fn real_glass_wgsl_validates_with_fragment_varying_capture_gate() {
    // Exercise the production optical path under explicitly divergent control
    // flow. All blur/dispersion taps must use LOD0, not implicit derivatives.
    let divergent = SHADER.replace(
        "if backdrop_enabled {",
        "if backdrop_enabled && in.position.x > 0.0 {",
    );
    assert_ne!(divergent, SHADER);
    validate_wgsl(&divergent).expect("glass sampling must allow per-fragment branches");
}

#[cfg(debug_assertions)]
#[test]
fn real_glass_wgsl_rejects_invalid_snell_normal_type() {
    // Negative control within the actual optical implementation, not a toy
    // shader: the validator must reach the new refract call and reject vec2 N.
    let invalid = SHADER.replace(
        "local_normal, 1.0 / ior)",
        "local_normal.xy, 1.0 / ior)",
    );
    assert_ne!(invalid, SHADER);
    assert!(validate_wgsl(&invalid).is_err());
}