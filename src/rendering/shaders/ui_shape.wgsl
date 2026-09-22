#import bevy_render::{
    view::View,
    globals::Globals,
}
#import bevy_ui::ui_vertex_output::UiVertexOutput

@group(0) @binding(0)
var<uniform> view: View;
@group(0) @binding(1)
var<uniform> globals: Globals;

const MAX_GRADIENT_STOPS: u32 = 4u;
const SHADOW_FALLOFF_LINEAR: f32 = 0.0;
const SHADOW_FALLOFF_SMOOTH: f32 = 1.0;
const SHADOW_FALLOFF_GAUSSIAN: f32 = 2.0;

struct GradientStopUniform {
    // linear RGBA, straight alpha
    color: vec4<f32>,
    // x = position [0,1], yzw reserved
    position_and_pad: vec4<f32>,
};

struct PaintUniform {
    // x = kind, y = stop_count, z = flags, w = extend mode
    kind_and_flags: vec4<f32>,
    // used by solid paint
    solid_color: vec4<f32>,
    // start.xy, end.xy for linear gradients in normalized local UV
    linear_points: vec4<f32>,
    // center.xy, radius.xy for radial gradients in normalized local UV
    radial_center_radius: vec4<f32>,
    // center.xy, angle_radians for angular gradients
    angular_center_angle: vec4<f32>,
    // fixed stop budget for initial implementation
    stops: array<GradientStopUniform, MAX_GRADIENT_STOPS>,
};

struct NoiseUniform {
    // x = enabled, y = kind, z = strength, w = scale_px
    params0: vec4<f32>,
    // x = seed, y = speed, z = animated flag, w = coordinate space
    params1: vec4<f32>,
    // x = time_seconds, y = target region, zw reserved
    params2: vec4<f32>,
};

struct OuterShadowUniform {
    // linear RGBA, straight alpha
    color: vec4<f32>,
    // x/y = offset, z = blur, w = spread in physical pixels
    offset_blur_spread: vec4<f32>,
    // x = opacity [0,1], y = falloff kind
    opacity_and_falloff: vec4<f32>,
};

struct OuterGlowUniform {
    // linear RGBA, straight alpha
    color: vec4<f32>,
    // x = blur, y = spread, z = opacity [0,1], w = falloff kind
    blur_spread_opacity_falloff: vec4<f32>,
};

struct InnerShadowUniform {
    // linear RGBA, straight alpha
    color: vec4<f32>,
    // x/y = offset, z = blur, w = spread in physical pixels
    offset_blur_spread: vec4<f32>,
    // x = opacity [0,1], y = falloff kind
    opacity_and_falloff: vec4<f32>,
};

struct UiShapeUniforms {
    // xy = physical size in pixels, z = shape kind, w = reserved
    size_and_kind: vec4<f32>,

    // top-left, top-right, bottom-right, bottom-left in physical pixels
    corner_radii: vec4<f32>,

    // fill and border paint descriptors
    fill_paint: PaintUniform,
    border_paint: PaintUniform,
    noise: NoiseUniform,

    // top, right, bottom, left in physical pixels
    border_widths: vec4<f32>,

    // left, top, right, bottom effect expansion in physical pixels
    effect_bounds: vec4<f32>,

    outer_shadow: OuterShadowUniform,
    outer_glow: OuterGlowUniform,
    inner_shadow: InnerShadowUniform,

    focus_primary_paint: PaintUniform,
    focus_secondary_paint: PaintUniform,
    // x = width px, y = offset px, z = opacity [0,1], w = enabled
    focus_primary_metrics: vec4<f32>,
    // x = width px, y = offset px, z = opacity [0,1], w = enabled
    focus_secondary_metrics: vec4<f32>,
    // x = placement (0 outside, 1 center, 2 inside), y = glow enabled
    focus_flags: vec4<f32>,
    focus_glow: OuterGlowUniform,

    // Clip/mask payloads. Shape masks are implemented via the same rounded-rect SDF.
    clip_kind: f32,
    clip_opacity: f32,
    clip_radii: vec4<f32>,
    mask_kind: f32,
    mask_opacity: f32,
    mask_radii: vec4<f32>,
    // x = enabled, y = blur_px, z = brightness, w = saturation
    backdrop_params0: vec4<f32>,
    // x = contrast, y = tint_opacity, z = debug mode, w = quality tier
    backdrop_params1: vec4<f32>,
    // linear RGBA tint color
    backdrop_tint: vec4<f32>,
    // min.xy + max.xy in normalized UVs for backdrop sampling
    backdrop_uv_rect: vec4<f32>,
    // Optical layer ABI; CPU converts logical defaults (3px / 4.5px) to physical px.
    // enabled, thickness px, bezel px, refractive index [1, 1.8]
    glass_optics: vec4<f32>,
    // specular intensity (0.28), width px (0.8 logical), fresnel (0.14), dispersion (0.018)
    glass_light: vec4<f32>,
    // profile: convex/squircle/concave/lip, press [0,1], screen light direction.xy
    glass_state: vec4<f32>,
    // debug: final/displacement/bezel/normal/fresnel/specular/UV, reserved.yzw
    glass_debug: vec4<f32>,
    // Local-physical -> screen-physical linear transform columns; default (1,0,0,1).
    surface_axes: vec4<f32>,
    // renderer debug mode
    // 0=final, 1=sdf, 2=border coverage, 3=gradient uv, 4=border paint
    // 5=noise raw, 6=noise coords, 7=noise strength, 8=noise modulation
    // 9=focus coverage, 10=focus distance
    debug_view: f32,
};

@group(1) @binding(0)
var<uniform> uniforms: UiShapeUniforms;

@group(1) @binding(1)
var backdrop_texture: texture_2d<f32>;

@group(1) @binding(2)
var backdrop_sampler: sampler;

@vertex
fn vertex(
    @location(0) vertex_position: vec3<f32>,
    @location(1) vertex_uv: vec2<f32>,
    @location(2) size: vec2<f32>,
    @location(3) border_widths: vec4<f32>,
    @location(4) border_radius: vec4<f32>,
) -> UiVertexOutput {
    var out: UiVertexOutput;
    let extents = max(uniforms.effect_bounds, vec4<f32>(0.0));
    let expand = vec2<f32>(
        mix(-extents.x, extents.z, vertex_uv.x),
        mix(-extents.y, extents.w, vertex_uv.y),
    );
    // Expansion is local geometry, not screen-aligned padding (rotation/scale/shear).
    let screen_expand = uniforms.surface_axes.xy * expand.x + uniforms.surface_axes.zw * expand.y;
    let expanded_position = vertex_position + vec3<f32>(screen_expand, 0.0);

    out.uv = vertex_uv;
    out.position = view.clip_from_world * vec4<f32>(expanded_position, 1.0);
    out.size = size;
    out.border_widths = border_widths;
    out.border_radius = border_radius;
    return out;
}

fn stop_position(stop: GradientStopUniform) -> f32 {
    return clamp(stop.position_and_pad.x, 0.0, 1.0);
}

fn to_premul(color: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(color.rgb * color.a, color.a);
}

fn from_premul(color: vec4<f32>) -> vec4<f32> {
    if color.a <= 1e-5 {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    return vec4<f32>(color.rgb / color.a, color.a);
}

fn sample_gradient_stops(paint: PaintUniform, t_raw: f32) -> vec4<f32> {
    let stop_count = u32(clamp(paint.kind_and_flags.y, 0.0, f32(MAX_GRADIENT_STOPS)));

    if stop_count == 0u {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
    if stop_count == 1u {
        return paint.stops[0].color;
    }

    let t = clamp(t_raw, 0.0, 1.0);
    var prev = paint.stops[0];
    let first_pos = stop_position(prev);
    if t <= first_pos {
        return prev.color;
    }

    let last = paint.stops[stop_count - 1u];
    let last_pos = stop_position(last);
    if t >= last_pos {
        return last.color;
    }

    var i: u32 = 1u;
    loop {
        if i >= stop_count {
            break;
        }

        let next = paint.stops[i];
        let prev_pos = stop_position(prev);
        let next_pos = stop_position(next);

        if t <= next_pos {
            let span = max(next_pos - prev_pos, 1e-6);
            let f = clamp((t - prev_pos) / span, 0.0, 1.0);
            let a = to_premul(prev.color);
            let b = to_premul(next.color);
            return from_premul(mix(a, b, f));
        }

        prev = next;
        i = i + 1u;
    }

    return last.color;
}

fn linear_gradient_t(uv: vec2<f32>, paint: PaintUniform) -> f32 {
    let start = paint.linear_points.xy;
    let end = paint.linear_points.zw;
    let dir = end - start;
    let len2 = max(dot(dir, dir), 1e-6);
    return dot(uv - start, dir) / len2;
}

fn radial_gradient_t(uv: vec2<f32>, paint: PaintUniform) -> f32 {
    let center = paint.radial_center_radius.xy;
    let radius = max(paint.radial_center_radius.zw, vec2<f32>(1e-4, 1e-4));
    return length((uv - center) / radius);
}

fn angular_gradient_t(uv: vec2<f32>, paint: PaintUniform) -> f32 {
    let center = paint.angular_center_angle.xy;
    let offset = paint.angular_center_angle.z;
    let delta = uv - center;
    let angle = atan2(delta.y, delta.x) - offset;
    let tau = 6.283185307179586;
    return fract(angle / tau);
}

// Shimmer reuses the gradient descriptor as a tagged union: base in solid_color,
// highlight in angular_center_angle, timing/direction/phase/width in linear_points,
// softness/intensity in radial_center_radius. No additional buffers or passes.
fn evaluate_shimmer(uv: vec2<f32>, paint: PaintUniform) -> vec4<f32> {
    if paint.kind_and_flags.w < 0.5 {
        return paint.solid_color;
    }
    let progress = fract(globals.time / paint.linear_points.x + paint.linear_points.z);
    let half_width = paint.linear_points.w * 0.5;
    // Travel completely outside both edges before wrapping: invisible cycle seam.
    let center = mix(-half_width, 1.0 + half_width, progress);
    let x = select(uv.x, 1.0 - uv.x, paint.linear_points.y < 0.0);
    let distance = abs(x - center);
    let edge_start = half_width * (1.0 - paint.radial_center_radius.x);
    let strength = (1.0 - smoothstep(edge_start, half_width, distance)) * paint.radial_center_radius.y;
    return from_premul(mix(to_premul(paint.solid_color), to_premul(paint.angular_center_angle), strength));
}

fn evaluate_paint(uv: vec2<f32>, paint: PaintUniform) -> vec4<f32> {
    let kind = paint.kind_and_flags.x;

    if kind > 3.5 {
        return evaluate_shimmer(uv, paint);
    }

    if kind < 0.5 {
        return paint.solid_color;
    }

    var t = 0.0;
    if kind < 1.5 {
        t = linear_gradient_t(uv, paint);
    } else if kind < 2.5 {
        t = radial_gradient_t(uv, paint);
    } else {
        t = angular_gradient_t(uv, paint);
    }

    return sample_gradient_stops(paint, t);
}

fn dither_noise(pixel: vec2<f32>) -> f32 {
    return hash12(pixel) - 0.5;
}

fn hash12(p: vec2<f32>) -> f32 {
    let seed = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(seed) * 43758.5453123);
}

fn value_noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);

    let a = hash12(i + vec2<f32>(0.0, 0.0));
    let b = hash12(i + vec2<f32>(1.0, 0.0));
    let c = hash12(i + vec2<f32>(0.0, 1.0));
    let d = hash12(i + vec2<f32>(1.0, 1.0));

    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn noise_coordinates(local_px: vec2<f32>, time_seconds: f32) -> vec2<f32> {
    let scale = max(uniforms.noise.params0.w, 1e-3);
    let seed = uniforms.noise.params1.x;
    let animated = uniforms.noise.params1.z > 0.5;
    let speed = uniforms.noise.params1.y;
    var motion = vec2<f32>(0.0, 0.0);
    if animated {
        motion = vec2<f32>(0.37, 0.53) * (time_seconds * speed);
    }
    let seed_offset = vec2<f32>(seed * 17.0, seed * 53.0);
    return (local_px / scale) + seed_offset + motion;
}

fn sample_noise_value(local_px: vec2<f32>, time_seconds: f32) -> f32 {
    let coords = noise_coordinates(local_px, time_seconds);
    return value_noise(coords) * 2.0 - 1.0;
}

fn modulate_luminance_premul(color_premul: vec4<f32>, modulation: f32) -> vec4<f32> {
    if color_premul.a <= 1e-5 {
        return color_premul;
    }

    let straight = color_premul.rgb / color_premul.a;
    let factor = max(0.0, 1.0 + modulation);
    let adjusted = clamp(straight * factor, vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(adjusted * color_premul.a, color_premul.a);
}

fn corner_radius(local: vec2<f32>, radii: vec4<f32>) -> f32 {
    let right = local.x >= 0.0;
    let bottom = local.y >= 0.0;

    if !right && !bottom {
        return radii.x;
    }
    if right && !bottom {
        return radii.y;
    }
    if right && bottom {
        return radii.z;
    }
    return radii.w;
}

// Signed distance to a rounded rectangle centered at the origin.
// local.x grows right and local.y grows down.
// This quadrant-selected formula requires radii <= min(half_size).
// CSS-normalized radii alone are NOT sufficient: in a 100x100 box with
// TL=80 and the other corners zero, crossing x=0 near y=-49 switches from
// positive distance to negative distance. The large TL arc crosses quadrants.
// Keep the caller clamps until this SDF (including effect/clip/mask uses) is
// replaced with a distance function supporting full asymmetric CSS geometry.
fn sdf_rounded_rect(local: vec2<f32>, half_size: vec2<f32>, radii: vec4<f32>) -> f32 {
    let r = corner_radius(local, radii);
    let q = abs(local) - half_size + vec2<f32>(r, r);
    return length(max(q, vec2<f32>(0.0, 0.0))) + min(max(q.x, q.y), 0.0) - r;
}

// Border width interpolation for per-side inside borders.
//
// We weight each side by inverse distance to that side in the local box frame.
// This yields smooth transitions around corners instead of hard side switches.
fn border_width_at(local: vec2<f32>, half_size: vec2<f32>, widths: vec4<f32>) -> f32 {
    let d_top = abs(local.y + half_size.y);
    let d_right = abs(half_size.x - local.x);
    let d_bottom = abs(half_size.y - local.y);
    let d_left = abs(local.x + half_size.x);

    let inv = vec4<f32>(
        1.0 / max(d_top, 1e-3),
        1.0 / max(d_right, 1e-3),
        1.0 / max(d_bottom, 1e-3),
        1.0 / max(d_left, 1e-3),
    );
    let weight_sum = max(dot(inv, vec4<f32>(1.0)), 1e-4);
    return dot(inv, widths) / weight_sum;
}

fn falloff_value(distance_outside: f32, blur: f32, aa: f32, kind: f32) -> f32 {
    let soft = max(max(blur, aa), 1e-4);
    let d = max(distance_outside, 0.0);

    if kind < 0.5 {
        return clamp(1.0 - d / soft, 0.0, 1.0);
    }

    if kind < 1.5 {
        return 1.0 - smoothstep(0.0, soft, d);
    }

    let sigma = max(soft * 0.5, 1e-4);
    let x = d / sigma;
    return exp(-0.5 * x * x);
}

fn apply_shadow(distance: f32, aa: f32, exterior_mask: f32) -> vec4<f32> {
    let blur = max(uniforms.outer_shadow.offset_blur_spread.z, 0.0);
    let spread = uniforms.outer_shadow.offset_blur_spread.w;
    let offset = uniforms.outer_shadow.offset_blur_spread.xy;
    let opacity = clamp(uniforms.outer_shadow.opacity_and_falloff.x, 0.0, 1.0);
    let falloff_kind = uniforms.outer_shadow.opacity_and_falloff.y;
    let color = clamp(uniforms.outer_shadow.color, vec4<f32>(0.0), vec4<f32>(1.0));

    if opacity <= 0.0 || color.a <= 0.0 {
        return vec4<f32>(0.0);
    }

    let outside = max(distance - spread, 0.0);
    let profile = falloff_value(outside, blur, aa, falloff_kind);
    let alpha = clamp(color.a * opacity * profile * exterior_mask, 0.0, 1.0);
    return vec4<f32>(color.rgb * alpha, alpha);
}

fn apply_glow(distance: f32, aa: f32, exterior_mask: f32) -> vec4<f32> {
    let blur = max(uniforms.outer_glow.blur_spread_opacity_falloff.x, 0.0);
    let spread = uniforms.outer_glow.blur_spread_opacity_falloff.y;
    let opacity = clamp(uniforms.outer_glow.blur_spread_opacity_falloff.z, 0.0, 1.0);
    let falloff_kind = uniforms.outer_glow.blur_spread_opacity_falloff.w;
    let color = clamp(uniforms.outer_glow.color, vec4<f32>(0.0), vec4<f32>(1.0));

    if opacity <= 0.0 || color.a <= 0.0 {
        return vec4<f32>(0.0);
    }

    let outside = max(distance - spread, 0.0);
    // Glow is intentionally a bit broader/softer than a comparable shadow.
    let profile = pow(falloff_value(outside, blur, aa, falloff_kind), 0.8);
    let alpha = clamp(color.a * opacity * profile * exterior_mask, 0.0, 1.0);
    return vec4<f32>(color.rgb * alpha, alpha);
}

fn focus_ring_bounds(placement: f32, width: f32, offset: f32) -> vec2<f32> {
    let safe_width = max(width, 0.0);
    let safe_offset = max(offset, 0.0);

    if placement < 0.5 {
        return vec2<f32>(safe_offset, safe_offset + safe_width);
    }

    if placement < 1.5 {
        let half_width = safe_width * 0.5;
        return vec2<f32>(safe_offset - half_width, safe_offset + half_width);
    }

    return vec2<f32>(-(safe_offset + safe_width), -safe_offset);
}

fn band_coverage(distance: f32, inner: f32, outer: f32, aa: f32) -> f32 {
    let enter = smoothstep(inner - aa, inner + aa, distance);
    let exit = 1.0 - smoothstep(outer - aa, outer + aa, distance);
    return clamp(enter * exit, 0.0, 1.0);
}

fn apply_focus_glow(
    distance: f32,
    ring_outer: f32,
    aa: f32,
    glow: OuterGlowUniform,
    mask: f32,
) -> vec4<f32> {
    let blur = max(glow.blur_spread_opacity_falloff.x, 0.0);
    let spread = max(glow.blur_spread_opacity_falloff.y, 0.0);
    let opacity = clamp(glow.blur_spread_opacity_falloff.z, 0.0, 1.0);
    let falloff_kind = glow.blur_spread_opacity_falloff.w;
    let color = clamp(glow.color, vec4<f32>(0.0), vec4<f32>(1.0));

    if opacity <= 0.0 || color.a <= 0.0 {
        return vec4<f32>(0.0);
    }

    let ring_distance = max(distance - ring_outer - spread, 0.0);
    let profile = pow(falloff_value(ring_distance, blur, aa, falloff_kind), 0.8);
    let alpha = clamp(color.a * opacity * profile * mask, 0.0, 1.0);
    return vec4<f32>(color.rgb * alpha, alpha);
}

fn apply_inner_shadow(
    local: vec2<f32>,
    half_size: vec2<f32>,
    radii: vec4<f32>,
    aa: f32,
    interior_mask: f32,
) -> vec4<f32> {
    let blur = max(uniforms.inner_shadow.offset_blur_spread.z, 0.0);
    let spread = uniforms.inner_shadow.offset_blur_spread.w;
    let offset = uniforms.inner_shadow.offset_blur_spread.xy;
    let opacity = clamp(uniforms.inner_shadow.opacity_and_falloff.x, 0.0, 1.0);
    let falloff_kind = uniforms.inner_shadow.opacity_and_falloff.y;
    let color = clamp(uniforms.inner_shadow.color, vec4<f32>(0.0), vec4<f32>(1.0));

    if opacity <= 0.0 || color.a <= 0.0 {
        return vec4<f32>(0.0);
    }

    let offset_distance = sdf_rounded_rect(local - offset, half_size, radii);
    // Positive spread moves the inner-shadow boundary inward from the edge.
    let inward_depth = max(-spread - offset_distance, 0.0);
    let profile = falloff_value(inward_depth, blur, aa, falloff_kind);
    let alpha = clamp(color.a * opacity * profile * interior_mask, 0.0, 1.0);
    return vec4<f32>(color.rgb * alpha, alpha);
}

fn apply_shape_mask(base: vec4<f32>, local: vec2<f32>, half_size: vec2<f32>, mask_radii: vec4<f32>, opacity: f32) -> vec4<f32> {
    let max_radius = max(min(half_size.x, half_size.y), 0.0);
    let safe_radii = clamp(mask_radii, vec4<f32>(0.0), vec4<f32>(max_radius));
    let distance = sdf_rounded_rect(local, half_size, safe_radii);
    let coverage = (1.0 - smoothstep(-max(fwidth(distance), 1e-4), max(fwidth(distance), 1e-4), distance));
    let alpha = clamp(base.a * coverage * opacity, 0.0, 1.0);
    return vec4<f32>(base.rgb * (alpha / max(base.a, 1e-4)), alpha);
}

fn composite_over(bottom: vec4<f32>, top: vec4<f32>) -> vec4<f32> {
    return top + bottom * (1.0 - top.a);
}

fn saturate(value: vec3<f32>, amount: f32) -> vec3<f32> {
    let luma = dot(value, vec3<f32>(0.2126, 0.7152, 0.0722));
    return mix(vec3<f32>(luma), value, amount);
}

fn contrast(value: vec3<f32>, amount: f32) -> vec3<f32> {
    return (value - vec3<f32>(0.5)) * amount + vec3<f32>(0.5);
}

// Capture and fragment position share screen physical pixels. Never reconstruct
// capture coordinates from local UV / an AABB: rotation, clipping and partially
// offscreen surfaces would stretch the image. Clamp only at the capture boundary.
// Explicit LOD permits per-fragment optical blur and debug branches without
// implicit-derivative uniform-control-flow requirements. Capture is premultiplied.
fn sample_backdrop_source(screen_uv: vec2<f32>) -> vec4<f32> {
    let half_texel = vec2<f32>(0.5) / vec2<f32>(textureDimensions(backdrop_texture));
    let uv = clamp(screen_uv, half_texel, vec2<f32>(1.0) - half_texel);
    return textureSampleLevel(backdrop_texture, backdrop_sampler, uv, 0.0);
}

fn sample_backdrop_blurred(screen_uv: vec2<f32>, blur_px: f32, quality: f32) -> vec4<f32> {
    if blur_px <= 1e-3 {
        return sample_backdrop_source(screen_uv);
    }

    let size = vec2<f32>(textureDimensions(backdrop_texture));
    let texel = vec2<f32>(1.0) / max(size, vec2<f32>(1.0));
    let uv = screen_uv;

    let radius = blur_px * texel;
    let axis_x = vec2<f32>(radius.x, 0.0);
    let axis_y = vec2<f32>(0.0, radius.y);
    let diag = radius;

    var accum = sample_backdrop_source(uv) * 0.227027;
    var total = 0.227027;

    var pair_weight_a = 0.1945946;
    if quality < 0.5 {
        pair_weight_a = 0.316216;
    }

    var pair_weight_b = 0.1216216;
    if quality < 1.5 {
        pair_weight_b = 0.070270;
    }

    accum += sample_backdrop_source(uv + axis_x) * pair_weight_a;
    accum += sample_backdrop_source(uv - axis_x) * pair_weight_a;
    accum += sample_backdrop_source(uv + axis_y) * pair_weight_a;
    accum += sample_backdrop_source(uv - axis_y) * pair_weight_a;
    total += pair_weight_a * 4.0;

    accum += sample_backdrop_source(uv + diag) * pair_weight_b;
    accum += sample_backdrop_source(uv - diag) * pair_weight_b;
    accum += sample_backdrop_source(uv + vec2<f32>(diag.x, -diag.y)) * pair_weight_b;
    accum += sample_backdrop_source(uv - vec2<f32>(diag.x, -diag.y)) * pair_weight_b;
    total += pair_weight_b * 4.0;

    if quality > 1.5 {
        let far = radius * 2.0;
        let far_weight = 0.054054;
        accum += sample_backdrop_source(uv + vec2<f32>(far.x, 0.0)) * far_weight;
        accum += sample_backdrop_source(uv - vec2<f32>(far.x, 0.0)) * far_weight;
        accum += sample_backdrop_source(uv + vec2<f32>(0.0, far.y)) * far_weight;
        accum += sample_backdrop_source(uv - vec2<f32>(0.0, far.y)) * far_weight;
        total += far_weight * 4.0;
    }

    return accum / max(total, 1e-4);
}

fn process_backdrop_color(sample: vec4<f32>) -> vec4<f32> {
    let brightness = clamp(uniforms.backdrop_params0.z, 0.0, 2.0);
    let saturation = clamp(uniforms.backdrop_params0.w, 0.0, 2.0);
    let contrast_value = clamp(uniforms.backdrop_params1.x, 0.0, 2.0);
    let tint_opacity = clamp(uniforms.backdrop_params1.y, 0.0, 1.0);
    let tint = clamp(uniforms.backdrop_tint, vec4<f32>(0.0), vec4<f32>(1.0));

    let alpha = clamp(sample.a, 0.0, 1.0);
    var straight_rgb = vec3<f32>(0.0);
    if alpha > 1e-5 {
        straight_rgb = sample.rgb / alpha;
    }

    var rgb = saturate(straight_rgb, saturation);
    rgb = contrast(rgb, contrast_value);
    rgb *= brightness;
    rgb = mix(rgb, tint.rgb, tint_opacity * tint.a);

    let clamped_rgb = clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(clamped_rgb * alpha, alpha);
}

// Optical height is parameterized by inward distance / bezel, not another SDF.
// Convex: sqrt(1-(1-x)^2); squircle: fourth root of 1-(1-x)^4.
// Inverse convex gives a recessed meniscus; a smooth blend gives a raised lip.
fn glass_profile_height(x_raw: f32, profile: f32) -> f32 {
    let x = clamp(x_raw, 0.0, 1.0);
    let q = 1.0 - x;
    let convex = sqrt(max(1.0 - q * q, 0.0));
    if profile < 0.5 {
        return convex;
    }
    if profile < 1.5 {
        return sqrt(sqrt(max(1.0 - q * q * q * q, 0.0)));
    }
    if profile < 2.5 {
        return 1.0 - convex;
    }
    return mix(convex, 1.0 - convex, smoothstep(0.15, 0.85, x));
}

fn glass_profile_derivative(x: f32, profile: f32) -> f32 {
    // Finite differences regularize the infinite endpoint derivative; the
    // final taper joins the bezel to an exactly flat, optically calm center.
    let lo = max(x - 0.01, 0.0);
    let hi = min(x + 0.01, 1.0);
    let derivative = (glass_profile_height(hi, profile) - glass_profile_height(lo, profile)) / max(hi - lo, 0.01);
    return clamp(derivative, -4.0, 4.0) * smoothstep(0.0, 0.15, 1.0 - x);
}

fn glass_sdf_normal(local: vec2<f32>, half_size: vec2<f32>, radii: vec4<f32>) -> vec2<f32> {
    let dx = vec2<f32>(0.5, 0.0);
    let dy = vec2<f32>(0.0, 0.5);
    let gradient = vec2<f32>(
        sdf_rounded_rect(local + dx, half_size, radii) - sdf_rounded_rect(local - dx, half_size, radii),
        sdf_rounded_rect(local + dy, half_size, radii) - sdf_rounded_rect(local - dy, half_size, radii),
    );
    return gradient / max(length(gradient), 1e-4);
}

fn glass_screen_slope(slope: vec2<f32>) -> vec2<f32> {
    // A^-T for normals, A for displacement. Normalizing the columns by a
    // common scale makes the singularity test relative, including reflections.
    let scale = max(length(uniforms.surface_axes.xy), length(uniforms.surface_axes.zw));
    let a = uniforms.surface_axes.xy / max(scale, 1e-4);
    let b = uniforms.surface_axes.zw / max(scale, 1e-4);
    let det = a.x * b.y - b.x * a.y;
    if scale < 1e-4 || abs(det) < 1e-4 {
        return vec2<f32>(0.0);
    }
    let transformed = vec2<f32>(b.y * slope.x - a.y * slope.y, -b.x * slope.x + a.x * slope.y) / (det * scale);
    return transformed;
}

struct GlassOpticalSample {
    displacement: vec2<f32>,
    bezel: f32,
    normal: vec3<f32>,
    fresnel: f32,
    specular: f32,
};

fn evaluate_glass_optics(local: vec2<f32>, half_size: vec2<f32>, radii: vec4<f32>, distance: f32) -> GlassOpticalSample {
    let thickness = max(uniforms.glass_optics.y, 0.0);
    let bezel = max(uniforms.glass_optics.z, 0.01);
    let depth = max(-distance, 0.0);
    let x = clamp(depth / bezel, 0.0, 1.0);
    let edge = 1.0 - smoothstep(0.0, 1.0, x);
    let press = clamp(uniforms.glass_state.y, 0.0, 1.0);
    let outward = glass_sdf_normal(local, half_size, radii);
    let derivative = glass_profile_derivative(x, uniforms.glass_state.x);
    // Press changes only optical slope/shift. Never rescale local geometry,
    // surface UV, paint, hit targets or child icons in this material pass.
    let slope = outward * clamp(derivative * thickness / bezel, -4.0, 4.0)
        * vec2<f32>(1.0 + 0.12 * press, 1.0 - 0.08 * press);
    let local_normal = normalize(vec3<f32>(slope, 1.0));
    let screen_normal = normalize(vec3<f32>(glass_screen_slope(slope), 1.0));
    let ior = clamp(uniforms.glass_optics.w, 1.0, 1.8);
    let ray = refract(vec3<f32>(0.0, 0.0, -1.0), local_normal, 1.0 / ior);
    var displacement = ray.xy * (thickness / max(-ray.z, 0.1));
    displacement += vec2<f32>(0.0, 0.06) * thickness * press * edge * (ior - 1.0);
    // Subtle refraction, bounded inside a fraction of the local bezel. Map
    // through the actual axes afterwards, so rotation/shear remain coherent.
    let limit = bezel * 0.35;
    displacement *= min(1.0, limit / max(length(displacement), 1e-4));
    let screen_displacement = uniforms.surface_axes.xy * displacement.x + uniforms.surface_axes.zw * displacement.y;
    let light_raw = uniforms.glass_state.zw;
    let light = light_raw / max(length(light_raw), 1e-4);
    let facing = max(dot(screen_normal.xy, light), 0.0);
    // Convert local depth to screen-normal distance for the physical highlight
    // width, also under nonuniform scale. Degenerate axes simply suppress light.
    let screen_depth = depth / max(length(glass_screen_slope(outward)), 1e-4);
    let width = max(uniforms.glass_light.y, 0.01);
    let band = exp(-0.5 * pow(min(screen_depth / width, 16.0), 2.0));
    var result: GlassOpticalSample;
    result.displacement = screen_displacement;
    result.bezel = edge;
    result.normal = screen_normal;
    result.fresnel = clamp(uniforms.glass_light.z, 0.0, 1.0) * pow(1.0 - screen_normal.z, 5.0) * edge;
    result.specular = clamp(uniforms.glass_light.x, 0.0, 1.0) * facing * facing * band * edge;
    return result;
}

fn sample_glass_backdrop(uv: vec2<f32>, texel: vec2<f32>, optics: GlassOpticalSample) -> vec4<f32> {
    // One shared blur kernel, progressively softer at the bezel. The CPU blur
    // is in physical pixels (default 1.4 logical -> ~2.5 logical at the edge).
    let center_blur = max(uniforms.backdrop_params0.y, 0.0);
    let blur = mix(center_blur, center_blur * (2.5 / 1.4), optics.bezel);
    var sample = sample_backdrop_blurred(uv, blur, uniforms.backdrop_params1.w);
    let chromatic = clamp(uniforms.glass_light.w, 0.0, 0.1);
    if chromatic > 1e-5 && optics.bezel > 1e-5 {
        // Reuse the blurred RGB; only TWO extra LOD0 taps, not three kernels.
        // Match each tap to the blurred alpha before differencing to avoid
        // colored halos when the captured scene contains translucent content.
        let offset = optics.displacement * chromatic * texel;
        let red = sample_backdrop_source(uv + offset);
        let blue = sample_backdrop_source(uv - offset);
        // Symmetric difference approximates +/- channel shifts about the
        // blurred RGB. Equal taps yield zero correction (no sharpening or
        // hue change at index=1, zero thickness, or the calm center).
        let delta = (red.rgb / max(red.a, 1e-5) - blue.rgb / max(blue.a, 1e-5)) * sample.a;
        let rgb = sample.rgb + vec3<f32>(delta.r, 0.0, -delta.b) * (0.5 * optics.bezel);
        sample = vec4<f32>(clamp(rgb, vec3<f32>(0.0), vec3<f32>(sample.a)), sample.a);
    }
    return sample;
}

fn glass_debug_color(optics: GlassOpticalSample, uv: vec2<f32>) -> vec3<f32> {
    let mode = uniforms.glass_debug.x;
    if mode > 5.5 { return vec3<f32>(clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0)), 0.0); }
    if mode > 4.5 { return vec3<f32>(optics.specular); }
    if mode > 3.5 { return vec3<f32>(optics.fresnel); }
    if mode > 2.5 { return optics.normal * 0.5 + vec3<f32>(0.5); }
    if mode > 1.5 { return vec3<f32>(optics.bezel); }
    return vec3<f32>(clamp(vec2<f32>(0.5) + optics.displacement / max(uniforms.glass_optics.z * 2.0, 0.01), vec2<f32>(0.0), vec2<f32>(1.0)), 0.5);
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let surface_size = max(uniforms.size_and_kind.xy, vec2<f32>(1.0, 1.0));
    let extents = max(uniforms.effect_bounds, vec4<f32>(0.0));
    let canvas_size = surface_size + vec2<f32>(extents.x + extents.z, extents.y + extents.w);
    let canvas_pos = in.uv * canvas_size;
    let shape_pos = canvas_pos - extents.xy;

    let local = shape_pos - surface_size * 0.5;
    let half_size = surface_size * 0.5;
    let surface_uv = shape_pos / surface_size;

    // Required by the quadrant SDF, not just a malformed-data guard. This
    // intentionally limits large asymmetric CSS radii; see sdf_rounded_rect.
    let max_radius = max(min(half_size.x, half_size.y), 0.0);
    let radii = clamp(uniforms.corner_radii, vec4<f32>(0.0), vec4<f32>(max_radius));

    var distance = 0.0;
    if uniforms.size_and_kind.z < 0.5 {
        distance = sdf_rounded_rect(local, half_size, radii);
    }

    // Analytic edge antialiasing using derivatives of signed distance.
    let aa = max(fwidth(distance), 1e-4);

    let border_width = border_width_at(local, half_size, max(uniforms.border_widths, vec4<f32>(0.0)));
    let inner_distance = distance + max(border_width, 0.0);

    // Outer boundary: outside -> shape interior.
    let outer_coverage = 1.0 - smoothstep(-aa, aa, distance);
    // Inner boundary: border -> fill.
    let fill_coverage = 1.0 - smoothstep(-aa, aa, inner_distance);
    let border_coverage = clamp(outer_coverage - fill_coverage, 0.0, 1.0);

    let fill_color = evaluate_paint(surface_uv, uniforms.fill_paint);
    let border_color = evaluate_paint(surface_uv, uniforms.border_paint);

    let fill_layer = to_premul(fill_color) * fill_coverage;
    let border_layer = to_premul(border_color) * border_coverage;

    let shape_exterior = clamp(1.0 - outer_coverage, 0.0, 1.0);
    let shadow_distance = sdf_rounded_rect(
        local - uniforms.outer_shadow.offset_blur_spread.xy,
        half_size,
        radii,
    );
    let shadow_layer = apply_shadow(shadow_distance, aa, shape_exterior);
    let glow_layer = apply_glow(distance, aa, shape_exterior);
    let inner_shadow_layer = apply_inner_shadow(local, half_size, radii, aa, fill_coverage);

    let clip_alpha = clamp(uniforms.clip_opacity, 0.0, 1.0);
    var clip_mask = 1.0;
    if uniforms.clip_kind < 0.5 {
        let clip_distance = sdf_rounded_rect(local, half_size, clamp(uniforms.clip_radii, vec4<f32>(0.0), vec4<f32>(max_radius)));
        let clip_coverage = 1.0 - smoothstep(-aa, aa, clip_distance);
        clip_mask = clip_coverage * clip_alpha;
    }

    let mask_alpha = clamp(uniforms.mask_opacity, 0.0, 1.0);
    var mask_coverage = 1.0;
    if uniforms.mask_kind < 0.5 {
        let mask_distance = sdf_rounded_rect(local, half_size, clamp(uniforms.mask_radii, vec4<f32>(0.0), vec4<f32>(max_radius)));
        let mask_value = 1.0 - smoothstep(-aa, aa, mask_distance);
        mask_coverage = mask_value * mask_alpha;
    }

    // Paint regions already include the outer shape's AA coverage. Apply only
    // the user clip/mask to them, but also shape-mask the rectangular backdrop.
    let surface_mask = clamp(clip_mask * mask_coverage, 0.0, 1.0);
    let effective_surface_coverage = outer_coverage * surface_mask;
    let render_debug_mode = uniforms.debug_view;

    let noise_enabled = uniforms.noise.params0.x > 0.5;
    let noise_strength = clamp(uniforms.noise.params0.z, 0.0, 1.0);
    let noise_time = max(uniforms.noise.params2.x, 0.0);
    var noise_value = 0.0;
    if noise_enabled && noise_strength > 1e-5 {
        noise_value = sample_noise_value(shape_pos, noise_time);
    }
    let noise_modulation = noise_value * noise_strength;

    let focus_placement = uniforms.focus_flags.x;

    let primary_ring = focus_ring_bounds(
        focus_placement,
        uniforms.focus_primary_metrics.x,
        uniforms.focus_primary_metrics.y,
    );
    let primary_enabled = uniforms.focus_primary_metrics.w > 0.5;
    var primary_coverage = 0.0;
    if primary_enabled {
        primary_coverage = band_coverage(distance, primary_ring.x, primary_ring.y, aa) * clamp(uniforms.focus_primary_metrics.z, 0.0, 1.0);
    }

    let secondary_ring = focus_ring_bounds(
        focus_placement,
        uniforms.focus_secondary_metrics.x,
        uniforms.focus_secondary_metrics.y,
    );
    let secondary_enabled = uniforms.focus_secondary_metrics.w > 0.5;
    var secondary_coverage = 0.0;
    if secondary_enabled {
        secondary_coverage = band_coverage(distance, secondary_ring.x, secondary_ring.y, aa) * clamp(uniforms.focus_secondary_metrics.z, 0.0, 1.0);
    }

    let focus_outer = max(primary_ring.y, secondary_ring.y);
    let focus_clip = clip_mask;

    if render_debug_mode > 9.5 {
        let centered = clamp(0.5 - (distance - focus_outer) / max(aa * 8.0, 1e-4), 0.0, 1.0);
        return vec4<f32>(centered, centered, centered, 1.0);
    }

    if render_debug_mode > 8.5 {
        let value = clamp((primary_coverage + secondary_coverage) * focus_clip, 0.0, 1.0);
        return vec4<f32>(value, value, value, 1.0);
    }

    if render_debug_mode > 7.5 {
        let value = clamp((noise_modulation + noise_strength) / max(noise_strength * 2.0, 1e-4), 0.0, 1.0);
        return vec4<f32>(vec3<f32>(value * effective_surface_coverage), 1.0);
    }

    if render_debug_mode > 6.5 {
        let value = clamp(noise_strength * effective_surface_coverage, 0.0, 1.0);
        return vec4<f32>(value, value, value, 1.0);
    }

    if render_debug_mode > 5.5 {
        let coords = noise_coordinates(shape_pos, noise_time);
        let uv_dbg = fract(coords * 0.125);
        return vec4<f32>(uv_dbg.x, uv_dbg.y, 0.0, 1.0);
    }

    if render_debug_mode > 4.5 {
        let value = clamp(noise_value * 0.5 + 0.5, 0.0, 1.0);
        return vec4<f32>(vec3<f32>(value * effective_surface_coverage), 1.0);
    }

    if render_debug_mode > 3.5 {
        let dbg = vec4<f32>(
            border_layer.rgb * surface_mask,
            border_layer.a * surface_mask,
        );
        if dbg.a <= 1e-5 {
            return vec4<f32>(0.0, 0.0, 0.0, 0.0);
        }
        return from_premul(dbg);
    }

    if render_debug_mode > 2.5 {
        let centered = surface_uv - vec2<f32>(0.5, 0.5);
        let angle_uv = fract((atan2(centered.y, centered.x) + 3.14159265359) / 6.28318530718);
        let uv_dbg = vec3<f32>(
            clamp(surface_uv.x, 0.0, 1.0),
            clamp(surface_uv.y, 0.0, 1.0),
            angle_uv,
        );
        return vec4<f32>(uv_dbg * effective_surface_coverage, 1.0);
    }

    if render_debug_mode > 1.5 {
        let value = clamp(border_coverage * surface_mask, 0.0, 1.0);
        return vec4<f32>(value, value, value, 1.0);
    }

    if render_debug_mode > 0.5 {
        let signed = clamp(0.5 - distance / max(aa * 8.0, 1e-4), 0.0, 1.0);
        let value = signed * effective_surface_coverage;
        return vec4<f32>(value, value, value, 1.0);
    }

    let backdrop_enabled = uniforms.backdrop_params0.x > 0.5;
    let backdrop_debug_mode = uniforms.backdrop_params1.z;

    if backdrop_debug_mode > 2.5 {
        let mask_value = effective_surface_coverage;
        return vec4<f32>(mask_value, mask_value, mask_value, 1.0);
    }

    var backdrop_layer = vec4<f32>(0.0);
    // No texture access at all when capture is unavailable/disabled, even if
    // glass is enabled. Ordinary paint, shadows and focus retain their order.
    if backdrop_enabled {
        let capture_size = max(vec2<f32>(textureDimensions(backdrop_texture)), vec2<f32>(1.0));
        let screen_uv = in.position.xy / capture_size;
        let texel = vec2<f32>(1.0) / capture_size;
        // Source/blur diagnostics show the un-refracted capture, before tint
        // and optics. It is ALREADY premultiplied; never multiply alpha twice.
        if backdrop_debug_mode > 1.5 {
            let dbg = sample_backdrop_blurred(screen_uv, max(uniforms.backdrop_params0.y, 0.0), uniforms.backdrop_params1.w);
            return from_premul(dbg * effective_surface_coverage);
        }
        if backdrop_debug_mode > 0.5 {
            let dbg = sample_backdrop_source(screen_uv);
            return from_premul(dbg * effective_surface_coverage);
        }
        var backdrop_processed = vec4<f32>(0.0);
        if uniforms.glass_optics.x > 0.5 {
            let optics = evaluate_glass_optics(local, half_size, radii, distance);
            let refracted_uv = screen_uv + optics.displacement * texel;
            if uniforms.glass_debug.x > 0.5 {
                let dbg = vec4<f32>(glass_debug_color(optics, refracted_uv), 1.0);
                return from_premul(dbg * effective_surface_coverage);
            }
            let optical_sample = sample_glass_backdrop(refracted_uv, texel, optics);
            backdrop_processed = process_backdrop_color(optical_sample);
            // White optical reflection, not a new shadow or opaque overlay.
            // Preserve capture alpha and add highlights after color processing.
            let reflection = clamp(optics.fresnel + optics.specular, 0.0, 1.0);
            backdrop_processed = vec4<f32>(mix(backdrop_processed.rgb, vec3<f32>(backdrop_processed.a), reflection), backdrop_processed.a);
        } else {
            backdrop_processed = process_backdrop_color(sample_backdrop_blurred(screen_uv, max(uniforms.backdrop_params0.y, 0.0), uniforms.backdrop_params1.w));
        }
        backdrop_layer = vec4<f32>(
            backdrop_processed.rgb * effective_surface_coverage,
            backdrop_processed.a * effective_surface_coverage,
        );
    } else if backdrop_debug_mode > 0.5 || uniforms.glass_debug.x > 0.5 {
        return vec4<f32>(0.0);
    }

    var masked_border = border_layer * surface_mask;
    var masked_fill = fill_layer * surface_mask;
    let masked_inner_shadow = inner_shadow_layer * surface_mask;

    var surface_layer = vec4<f32>(0.0);
    surface_layer = composite_over(surface_layer, backdrop_layer);

    if uniforms.noise.params2.y > 1.5 {
        masked_border = modulate_luminance_premul(masked_border, noise_modulation);
    }
    if uniforms.noise.params2.y > 0.5 && uniforms.noise.params2.y <= 1.5 {
        masked_fill = modulate_luminance_premul(masked_fill, noise_modulation);
    }

    // Fill and border partition coverage, rather than overlapping layers.
    // Source-over here would turn two opaque half-covered regions into 0.75
    // alpha (and bias their colors). Sum premultiplied region contributions
    // after region-specific noise, then composite the paint as one layer.
    let paint_layer = masked_border + masked_fill;
    surface_layer = composite_over(surface_layer, paint_layer);

    if uniforms.noise.params2.y < 0.5 {
        surface_layer = modulate_luminance_premul(surface_layer, noise_modulation);
    }

    var out_premul = vec4<f32>(0.0);
    out_premul = composite_over(out_premul, shadow_layer);
    out_premul = composite_over(out_premul, glow_layer);
    out_premul = composite_over(out_premul, surface_layer);
    out_premul = composite_over(out_premul, masked_inner_shadow);

    let use_dither = uniforms.fill_paint.kind_and_flags.z > 0.5 || uniforms.border_paint.kind_and_flags.z > 0.5;
    if use_dither && out_premul.a > 1e-5 {
        let local_pixel = shape_pos;
        let jitter = dither_noise(local_pixel) * (0.35 / 255.0);
        let rgb = clamp(
            out_premul.rgb + vec3<f32>(jitter) * out_premul.a,
            vec3<f32>(0.0),
            vec3<f32>(out_premul.a),
        );
        out_premul = vec4<f32>(rgb, out_premul.a);
    }

    let focus_primary_color = evaluate_paint(surface_uv, uniforms.focus_primary_paint);
    let focus_secondary_color = evaluate_paint(surface_uv, uniforms.focus_secondary_paint);

    let focus_primary_layer = to_premul(focus_primary_color) * (primary_coverage * focus_clip);
    let focus_secondary_layer = to_premul(focus_secondary_color) * (secondary_coverage * focus_clip);
    var focus_glow_layer = vec4<f32>(0.0);
    if uniforms.focus_flags.y > 0.5 {
        focus_glow_layer = apply_focus_glow(distance, focus_outer, aa, uniforms.focus_glow, focus_clip);
    }

    out_premul = composite_over(out_premul, focus_glow_layer);
    out_premul = composite_over(out_premul, focus_secondary_layer);
    out_premul = composite_over(out_premul, focus_primary_layer);

    if out_premul.a <= 1e-5 {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    let clamped_alpha = clamp(out_premul.a, 0.0, 1.0);
    let clamped_rgb = clamp(out_premul.rgb, vec3<f32>(0.0), vec3<f32>(clamped_alpha));
    return from_premul(vec4<f32>(clamped_rgb, clamped_alpha));
}
