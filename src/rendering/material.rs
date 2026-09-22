use bevy::{
    asset::Asset,
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderType},
    shader::ShaderRef,
};

use super::{
    backdrop::{Backdrop, BackdropDebugView, BackdropQuality},
    decoration::{FocusRing, FocusRingLayer, FocusRingPlacement},
    debug::UiRenderDebugView,
    effect::{Effects, InnerShadow, OuterGlow, OuterShadow, ShadowFalloff},
    noise::{Noise, NoiseKind, NoiseSpace, NoiseTarget},
    paint::{
        AngularGradient,
        GradientStop,
        LinearGradient,
        MAX_GRADIENT_STOPS,
        Paint,
        RadialGradient,
        normalize_stops,
    },
    sdf::{normalize_border_widths, normalize_corner_radii},
    shape::{RoundedRect, Shape},
    surface::Surface,
};

#[cfg(test)]
#[path = "shader_tests.rs"]
mod shader_tests;

const SHAPE_KIND_ROUNDED_RECT: f32 = 0.0;
const PAINT_KIND_SOLID: f32 = 0.0;
const PAINT_KIND_LINEAR: f32 = 1.0;
const PAINT_KIND_RADIAL: f32 = 2.0;
const PAINT_KIND_ANGULAR: f32 = 3.0;
const PAINT_FLAG_DITHERING: f32 = 1.0;
const SHADOW_FALLOFF_LINEAR: f32 = 0.0;
const SHADOW_FALLOFF_SMOOTH: f32 = 1.0;
const SHADOW_FALLOFF_GAUSSIAN: f32 = 2.0;
const BACKDROP_DEBUG_FINAL: f32 = 0.0;
const BACKDROP_DEBUG_SOURCE: f32 = 1.0;
const BACKDROP_DEBUG_BLURRED: f32 = 2.0;
const BACKDROP_DEBUG_MASK: f32 = 3.0;
const RENDER_DEBUG_FINAL: f32 = 0.0;
const RENDER_DEBUG_SDF: f32 = 1.0;
const RENDER_DEBUG_BORDER_COVERAGE: f32 = 2.0;
const RENDER_DEBUG_GRADIENT_UV: f32 = 3.0;
const RENDER_DEBUG_BORDER_PAINT: f32 = 4.0;
const RENDER_DEBUG_NOISE_RAW: f32 = 5.0;
const RENDER_DEBUG_NOISE_COORDS: f32 = 6.0;
const RENDER_DEBUG_NOISE_STRENGTH: f32 = 7.0;
const RENDER_DEBUG_NOISE_MODULATION: f32 = 8.0;
const RENDER_DEBUG_FOCUS_COVERAGE: f32 = 9.0;
const RENDER_DEBUG_FOCUS_DISTANCE: f32 = 10.0;
const BACKDROP_QUALITY_LOW: f32 = 0.0;
const BACKDROP_QUALITY_MEDIUM: f32 = 1.0;
const BACKDROP_QUALITY_HIGH: f32 = 2.0;
const NOISE_KIND_GRAIN: f32 = 0.0;
const NOISE_SPACE_LOCAL: f32 = 0.0;
const NOISE_TARGET_SURFACE: f32 = 0.0;
const NOISE_TARGET_FILL: f32 = 1.0;
const NOISE_TARGET_BORDER: f32 = 2.0;
const FOCUS_PLACEMENT_OUTSIDE: f32 = 0.0;
const FOCUS_PLACEMENT_CENTER: f32 = 1.0;
const FOCUS_PLACEMENT_INSIDE: f32 = 2.0;

// Encoding limits, not new authoring constraints. Keep spatial values within
// one million pixels/UV units and allow up to 64x DPI/UI zoom. Bound operands
// before multiplication: clamping only the result cannot repair infinity * 0.
const MAX_RENDER_EXTENT: f32 = 1_048_576.0;
const MAX_SCALE_FACTOR: f32 = 64.0;

#[derive(Clone, Copy, Debug, ShaderType, PartialEq)]
pub struct NoiseUniform {
    /// x = enabled, y = kind, z = strength, w = scale_px
    pub params0: Vec4,
    /// x = seed, y = speed, z = animated flag, w = coordinate space
    pub params1: Vec4,
    /// x = time_seconds, y = target region, zw reserved
    pub params2: Vec4,
}

impl Default for NoiseUniform {
    fn default() -> Self {
        Self {
            params0: Vec4::new(0.0, NOISE_KIND_GRAIN, 0.0, 24.0),
            params1: Vec4::new(0.0, 0.0, 0.0, NOISE_SPACE_LOCAL),
            params2: Vec4::new(0.0, NOISE_TARGET_SURFACE, 0.0, 0.0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BackdropSampleRegion {
    pub min_uv: Vec2,
    pub max_uv: Vec2,
}

impl Default for BackdropSampleRegion {
    fn default() -> Self {
        Self {
            min_uv: Vec2::ZERO,
            max_uv: Vec2::ONE,
        }
    }
}

#[derive(Clone, Copy, Debug, ShaderType, PartialEq)]
pub struct GradientStopUniform {
    /// Linear RGBA in straight-alpha representation.
    pub color: Vec4,
    /// x = stop position [0, 1], yzw reserved.
    pub position_and_pad: Vec4,
}

impl Default for GradientStopUniform {
    fn default() -> Self {
        Self {
            color: Vec4::ZERO,
            position_and_pad: Vec4::ZERO,
        }
    }
}

#[derive(Clone, Copy, Debug, ShaderType, PartialEq)]
pub struct PaintUniform {
    /// x = paint kind, y = stop count, z = flags, w = extend mode (0 = clamp)
    pub kind_and_flags: Vec4,
    /// Linear RGBA for solid paints (straight alpha).
    pub solid_color: Vec4,
    /// Linear start/end in normalized local UV: start.xy, end.xy.
    pub linear_points: Vec4,
    /// Radial center/radius in normalized local UV: center.xy, radius.xy.
    pub radial_center_radius: Vec4,
    /// Angular center + offset angle: center.xy, angle_radians, reserved.
    pub angular_center_angle: Vec4,
    /// Fixed-size GPU payload for the initial implementation.
    pub stops: [GradientStopUniform; MAX_GRADIENT_STOPS],
}

impl Default for PaintUniform {
    fn default() -> Self {
        Self {
            kind_and_flags: Vec4::new(PAINT_KIND_SOLID, 0.0, 0.0, 0.0),
            solid_color: Vec4::ZERO,
            linear_points: Vec4::new(0.0, 0.5, 1.0, 0.5),
            radial_center_radius: Vec4::new(0.5, 0.5, 0.5, 0.5),
            angular_center_angle: Vec4::new(0.5, 0.5, 0.0, 0.0),
            stops: [GradientStopUniform::default(); MAX_GRADIENT_STOPS],
        }
    }
}

#[derive(Clone, Copy, Debug, ShaderType, PartialEq)]
pub struct OuterShadowUniform {
    /// Linear RGBA in straight-alpha representation.
    pub color: Vec4,
    /// x/y = offset in physical pixels, z = blur in physical pixels, w = spread in physical pixels.
    pub offset_blur_spread: Vec4,
    /// x = opacity [0, 1], y = falloff kind discriminator, zw reserved.
    pub opacity_and_falloff: Vec4,
}

impl Default for OuterShadowUniform {
    fn default() -> Self {
        Self {
            color: Vec4::ZERO,
            offset_blur_spread: Vec4::ZERO,
            opacity_and_falloff: Vec4::new(0.0, SHADOW_FALLOFF_SMOOTH, 0.0, 0.0),
        }
    }
}

#[derive(Clone, Copy, Debug, ShaderType, PartialEq)]
pub struct OuterGlowUniform {
    /// Linear RGBA in straight-alpha representation.
    pub color: Vec4,
    /// x = blur in physical pixels, y = spread in physical pixels, z = opacity [0,1], w = falloff kind.
    pub blur_spread_opacity_falloff: Vec4,
}

#[derive(Clone, Copy, Debug, ShaderType, PartialEq)]
pub struct InnerShadowUniform {
    /// Linear RGBA in straight-alpha representation.
    pub color: Vec4,
    /// x/y = offset in physical pixels, z = blur in physical pixels, w = spread in physical pixels.
    pub offset_blur_spread: Vec4,
    /// x = opacity [0, 1], y = falloff kind discriminator, zw reserved.
    pub opacity_and_falloff: Vec4,
}

impl Default for InnerShadowUniform {
    fn default() -> Self {
        Self {
            color: Vec4::ZERO,
            offset_blur_spread: Vec4::ZERO,
            opacity_and_falloff: Vec4::new(0.0, SHADOW_FALLOFF_SMOOTH, 0.0, 0.0),
        }
    }
}

impl Default for OuterGlowUniform {
    fn default() -> Self {
        Self {
            color: Vec4::ZERO,
            blur_spread_opacity_falloff: Vec4::new(0.0, 0.0, 0.0, SHADOW_FALLOFF_GAUSSIAN),
        }
    }
}

/// Rust <-> WGSL uniform contract for `assets/shaders/ui_shape.wgsl`.
///
/// All values are in physical pixels except color:
/// - `size_and_kind.xy`: width/height in physical pixels
/// - `size_and_kind.z`: shape kind discriminator
/// - `corner_radii`: top-left, top-right, bottom-right, bottom-left (physical px)
/// - `fill_paint`: fill paint payload
/// - `border_paint`: border paint payload
/// - `border_widths`: top, right, bottom, left (physical px)
/// - `effect_bounds`: left, top, right, bottom visual expansion (physical px)
/// - `outer_shadow`: outer shadow payload
/// - `outer_glow`: outer glow payload
/// - `inner_shadow`: inner shadow payload
#[derive(Clone, Copy, Debug, ShaderType, PartialEq)]
pub struct UiShapeUniform {
    pub size_and_kind: Vec4,
    pub corner_radii: Vec4,
    pub fill_paint: PaintUniform,
    pub border_paint: PaintUniform,
    pub noise: NoiseUniform,
    pub border_widths: Vec4,
    pub effect_bounds: Vec4,
    pub outer_shadow: OuterShadowUniform,
    pub outer_glow: OuterGlowUniform,
    pub inner_shadow: InnerShadowUniform,
    pub focus_primary_paint: PaintUniform,
    pub focus_secondary_paint: PaintUniform,
    /// x = width px, y = offset px, z = opacity [0,1], w = enabled
    pub focus_primary_metrics: Vec4,
    /// x = width px, y = offset px, z = opacity [0,1], w = enabled
    pub focus_secondary_metrics: Vec4,
    /// x = placement (0 outside, 1 center, 2 inside), y = glow enabled
    pub focus_flags: Vec4,
    pub focus_glow: OuterGlowUniform,
    pub clip_kind: f32,
    pub clip_opacity: f32,
    pub clip_radii: Vec4,
    pub mask_kind: f32,
    pub mask_opacity: f32,
    pub mask_radii: Vec4,
    /// x = enabled, y = blur in physical px, z = brightness, w = saturation
    pub backdrop_params0: Vec4,
    /// x = contrast, y = tint opacity, z = debug mode, w = quality
    pub backdrop_params1: Vec4,
    /// Linear RGBA tint in straight alpha.
    pub backdrop_tint: Vec4,
    /// Backdrop sampling rectangle in normalized source UVs: min.xy max.xy.
    pub backdrop_uv_rect: Vec4,
    /// enabled, thickness px, bezel px, refractive index
    pub glass_optics: Vec4,
    /// specular intensity/width px, Fresnel strength, chromatic separation
    pub glass_light: Vec4,
    /// profile, normalized press, light direction xy
    pub glass_state: Vec4,
    /// optical debug mode (0 final, 1 displacement, 2 bezel, 3 normal, 4 Fresnel, 5 specular, 6 UV)
    pub glass_debug: Vec4,
    /// UiGlobalTransform linear columns: maps local physical pixels to viewport pixels.
    pub surface_axes: Vec4,
    /// Renderer-level debug view mode.
    pub debug_view: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct UiShapeMaterial {
    #[uniform(0)]
    pub uniforms: UiShapeUniform,
    #[texture(1)]
    #[sampler(2)]
    pub backdrop_texture: Handle<Image>,
}

impl UiMaterial for UiShapeMaterial {
    fn vertex_shader() -> ShaderRef {
        "embedded://beverly/rendering/shaders/ui_shape.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "embedded://beverly/rendering/shaders/ui_shape.wgsl".into()
    }
}

#[derive(Component, Debug, Clone)]
pub struct SurfaceMaterialHandle(pub Handle<UiShapeMaterial>);

impl SurfaceMaterialHandle {
    #[allow(dead_code)]
    pub fn handle(&self) -> Handle<UiShapeMaterial> {
        self.0.clone()
    }
}

#[allow(dead_code)]
pub fn build_shape_uniform(
    surface: &Surface,
    logical_size: Vec2,
    scale_factor: f32,
    sample_region: BackdropSampleRegion,
    backdrop_enabled: bool,
    debug_view: BackdropDebugView,
) -> UiShapeUniform {
    build_shape_uniform_with_debug_and_time(
        surface,
        logical_size,
        scale_factor,
        sample_region,
        backdrop_enabled,
        debug_view,
        UiRenderDebugView::Final,
        0.0,
        false,
    )
}

#[allow(dead_code)]
pub fn build_shape_uniform_with_debug(
    surface: &Surface,
    logical_size: Vec2,
    scale_factor: f32,
    sample_region: BackdropSampleRegion,
    backdrop_enabled: bool,
    debug_view: BackdropDebugView,
    render_debug_view: UiRenderDebugView,
) -> UiShapeUniform {
    build_shape_uniform_with_debug_and_time(
        surface,
        logical_size,
        scale_factor,
        sample_region,
        backdrop_enabled,
        debug_view,
        render_debug_view,
        0.0,
        false,
    )
}

/// Encodes bounded, finite geometry and standalone opacity fields, including
/// direct public-field inputs. NaN/negative sizes and scales become zero;
/// positive infinity saturates. Logical/physical extents are at most 1,048,576
/// pixels and scale is at most 64x; ordinary values retain their existing math.
/// This is not a guarantee for every possible authored `Color` float: existing
/// color-space conversion and post-conversion RGBA sanitization are unchanged.
pub fn build_shape_uniform_with_debug_and_time(
    surface: &Surface,
    logical_size: Vec2,
    scale_factor: f32,
    sample_region: BackdropSampleRegion,
    backdrop_enabled: bool,
    debug_view: BackdropDebugView,
    render_debug_view: UiRenderDebugView,
    noise_time_seconds: f32,
    reduce_effects: bool,
) -> UiShapeUniform {
    let scale_factor = scale_factor.max(0.0).min(MAX_SCALE_FACTOR);
    // Cap before normalization so radii and borders still fit the encoded size,
    // including when an extreme scale would otherwise overflow physical units.
    let max_logical_extent = MAX_RENDER_EXTENT / scale_factor.max(1.0);
    let logical_size = Vec2::new(
        logical_size.x.max(0.0).min(max_logical_extent),
        logical_size.y.max(0.0).min(max_logical_extent),
    );
    let physical_size = logical_size * scale_factor;

    let (shape_kind, corner_radii) = match surface.shape {
        Shape::RoundedRect(RoundedRect { radii }) => {
            let normalized = normalize_corner_radii(radii, logical_size);
            (SHAPE_KIND_ROUNDED_RECT, normalized.as_vec4() * scale_factor)
        }
    };

    let fill_paint = encode_paint(&surface.fill);
    let noise = encode_noise(surface.noise, noise_time_seconds, reduce_effects);

    let (border_paint, border_widths) = if let Some(border) = &surface.border {
        let normalized = normalize_border_widths(border.width, logical_size);
        (encode_paint(&border.paint), normalized.as_vec4() * scale_factor)
    } else {
        (PaintUniform::default(), Vec4::ZERO)
    };

    let (outer_shadow, outer_glow, inner_shadow, effect_bounds) =
        encode_effects(&surface.effects, scale_factor);
    let (focus_primary_paint, focus_secondary_paint, focus_primary_metrics, focus_secondary_metrics, focus_flags, focus_glow, focus_bounds) =
        encode_focus_ring(surface.decorations.focus_ring.as_ref(), scale_factor, reduce_effects);
    let effect_bounds = Vec4::new(
        effect_bounds.x.max(focus_bounds.x),
        effect_bounds.y.max(focus_bounds.y),
        effect_bounds.z.max(focus_bounds.z),
        effect_bounds.w.max(focus_bounds.w),
    );

    let clip = surface
        .clip
        .map(|clip| encode_clip(clip, logical_size, scale_factor))
        .unwrap_or_default();
    let mask = surface
        .mask
        .map(|mask| encode_mask(mask, logical_size, scale_factor))
        .unwrap_or_default();
    let backdrop = encode_backdrop(
        surface.backdrop,
        scale_factor,
        sample_region,
        backdrop_enabled,
        debug_view,
    );

    UiShapeUniform {
        size_and_kind: Vec4::new(physical_size.x, physical_size.y, shape_kind, 0.0),
        corner_radii,
        fill_paint,
        border_paint,
        noise,
        border_widths,
        effect_bounds,
        outer_shadow,
        outer_glow,
        inner_shadow,
        focus_primary_paint,
        focus_secondary_paint,
        focus_primary_metrics,
        focus_secondary_metrics,
        focus_flags,
        focus_glow,
        clip_kind: clip.kind,
        clip_opacity: clip.opacity,
        clip_radii: clip.radii,
        mask_kind: mask.kind,
        mask_opacity: mask.opacity,
        mask_radii: mask.radii,
        backdrop_params0: backdrop.params0,
        backdrop_params1: backdrop.params1,
        backdrop_tint: backdrop.tint,
        backdrop_uv_rect: backdrop.uv_rect,
        glass_optics: surface.backdrop.and_then(|b| b.liquid_glass)
            .filter(|g| g.enabled && backdrop_enabled && !reduce_effects)
            .map(|g| { let g = g.sanitized(); Vec4::new(1.0, g.thickness * scale_factor,
                g.bezel_width * scale_factor, g.refractive_index) }).unwrap_or(Vec4::ZERO),
        glass_light: surface.backdrop.and_then(|b| b.liquid_glass).map(|g| {
            let g = g.sanitized(); Vec4::new(g.specular_intensity,
                g.specular_width * scale_factor, g.fresnel, g.chromatic_aberration)
        }).unwrap_or(Vec4::ZERO),
        glass_state: surface.backdrop.and_then(|b| b.liquid_glass).map(|g| {
            let g = g.sanitized();
            let profile = match g.profile {
                super::GlassProfile::Convex => 0.0, super::GlassProfile::Squircle => 1.0,
                super::GlassProfile::Concave => 2.0, super::GlassProfile::Lip => 3.0,
            };
            Vec4::new(profile, g.press_amount, g.light_direction.x, g.light_direction.y)
        }).unwrap_or(Vec4::ZERO),
        glass_debug: Vec4::ZERO,
        surface_axes: Vec4::new(1.0, 0.0, 0.0, 1.0),
        debug_view: encode_render_debug(render_debug_view),
    }
}

fn encode_render_debug(mode: UiRenderDebugView) -> f32 {
    match mode {
        UiRenderDebugView::Final => RENDER_DEBUG_FINAL,
        UiRenderDebugView::Sdf => RENDER_DEBUG_SDF,
        UiRenderDebugView::BorderCoverage => RENDER_DEBUG_BORDER_COVERAGE,
        UiRenderDebugView::GradientUv => RENDER_DEBUG_GRADIENT_UV,
        UiRenderDebugView::BorderPaint => RENDER_DEBUG_BORDER_PAINT,
        UiRenderDebugView::NoiseRaw => RENDER_DEBUG_NOISE_RAW,
        UiRenderDebugView::NoiseCoords => RENDER_DEBUG_NOISE_COORDS,
        UiRenderDebugView::NoiseStrength => RENDER_DEBUG_NOISE_STRENGTH,
        UiRenderDebugView::NoiseModulation => RENDER_DEBUG_NOISE_MODULATION,
        UiRenderDebugView::FocusCoverage => RENDER_DEBUG_FOCUS_COVERAGE,
        UiRenderDebugView::FocusDistance => RENDER_DEBUG_FOCUS_DISTANCE,
    }
}

fn encode_focus_ring(
    focus_ring: Option<&FocusRing>,
    scale_factor: f32,
    reduce_effects: bool,
) -> (PaintUniform, PaintUniform, Vec4, Vec4, Vec4, OuterGlowUniform, Vec4) {
    let Some(focus_ring) = focus_ring else {
        return (
            PaintUniform::default(),
            PaintUniform::default(),
            Vec4::ZERO,
            Vec4::ZERO,
            Vec4::ZERO,
            OuterGlowUniform::default(),
            Vec4::ZERO,
        );
    };

    let focus = focus_ring.clone().sanitized();
    let primary = encode_focus_layer(&focus.primary, scale_factor);
    let secondary = focus
        .secondary
        .as_ref()
        .map(|layer| encode_focus_layer(layer, scale_factor))
        .unwrap_or((PaintUniform::default(), Vec4::ZERO));

    let placement = encode_focus_placement(focus.placement);
    let glow = if reduce_effects {
        OuterGlowUniform::default()
    } else {
        focus
            .glow
            .map(|value| encode_outer_glow(value.sanitized(), scale_factor))
            .unwrap_or_default()
    };

    let has_glow = if glow.blur_spread_opacity_falloff.z > 1e-5 && glow.color.w > 1e-5 {
        1.0
    } else {
        0.0
    };

    let primary_outer = focus_outer_extent_px(placement, primary.1.x, primary.1.y);
    let secondary_outer = focus_outer_extent_px(placement, secondary.1.x, secondary.1.y);
    let mut outer_extent = primary_outer.max(secondary_outer);
    if has_glow > 0.5 {
        let blur = glow.blur_spread_opacity_falloff.x;
        let spread = glow.blur_spread_opacity_falloff.y.max(0.0);
        let falloff = glow.blur_spread_opacity_falloff.w;
        outer_extent += spread + shadow_falloff_extent_from_code(blur, falloff) + 1.0;
    }

    (
        primary.0,
        secondary.0,
        primary.1,
        secondary.1,
        Vec4::new(placement, has_glow, 0.0, 0.0),
        glow,
        Vec4::splat(outer_extent.max(0.0).min(MAX_RENDER_EXTENT)),
    )
}

fn encode_focus_layer(layer: &FocusRingLayer, scale_factor: f32) -> (PaintUniform, Vec4) {
    let paint = encode_paint(&layer.paint);
    let width = layer.width.max(0.0) * scale_factor;
    let offset = layer.offset.max(0.0) * scale_factor;
    let opacity = layer.opacity.clamp(0.0, 1.0);
    let enabled = if width > 1e-5 && opacity > 1e-5 { 1.0 } else { 0.0 };
    (paint, Vec4::new(width, offset, opacity, enabled))
}

fn encode_focus_placement(placement: FocusRingPlacement) -> f32 {
    match placement {
        FocusRingPlacement::Outside => FOCUS_PLACEMENT_OUTSIDE,
        FocusRingPlacement::Center => FOCUS_PLACEMENT_CENTER,
        FocusRingPlacement::Inside => FOCUS_PLACEMENT_INSIDE,
    }
}

fn focus_outer_extent_px(placement_code: f32, width: f32, offset: f32) -> f32 {
    if width <= 0.0 {
        return 0.0;
    }

    if placement_code < 0.5 {
        offset.max(0.0) + width
    } else if placement_code < 1.5 {
        (offset.max(0.0) + width * 0.5).max(0.0)
    } else {
        0.0
    }
}

fn shadow_falloff_extent_from_code(blur: f32, falloff_code: f32) -> f32 {
    let blur = blur.max(0.0);
    if falloff_code < 0.5 {
        blur
    } else if falloff_code < 1.5 {
        blur * 1.25
    } else {
        blur * 3.0
    }
}

fn encode_noise(noise: Option<Noise>, time_seconds: f32, reduce_effects: bool) -> NoiseUniform {
    let mut payload = NoiseUniform::default();

    let Some(noise) = noise.map(Noise::sanitized) else {
        return payload;
    };

    let strength = if reduce_effects { 0.0 } else { noise.strength };
    let enabled = if strength > 1e-5 { 1.0 } else { 0.0 };

    payload.params0 = Vec4::new(
        enabled,
        encode_noise_kind(noise.kind),
        strength,
        noise.scale,
    );
    payload.params1 = Vec4::new(
        noise.seed.clamp(-MAX_RENDER_EXTENT, MAX_RENDER_EXTENT),
        noise.speed,
        if noise.animated { 1.0 } else { 0.0 },
        encode_noise_space(noise.space),
    );
    payload.params2 = Vec4::new(
        sanitize_finite(time_seconds, 0.0).clamp(0.0, MAX_RENDER_EXTENT),
        encode_noise_target(noise.target),
        0.0,
        0.0,
    );

    payload
}

fn encode_noise_kind(kind: NoiseKind) -> f32 {
    match kind {
        NoiseKind::Grain => NOISE_KIND_GRAIN,
    }
}

fn encode_noise_space(space: NoiseSpace) -> f32 {
    match space {
        NoiseSpace::Local => NOISE_SPACE_LOCAL,
    }
}

fn encode_noise_target(target: NoiseTarget) -> f32 {
    match target {
        NoiseTarget::Surface => NOISE_TARGET_SURFACE,
        NoiseTarget::Fill => NOISE_TARGET_FILL,
        NoiseTarget::Border => NOISE_TARGET_BORDER,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ClipMaskPayload {
    kind: f32,
    opacity: f32,
    radii: Vec4,
}

impl Default for ClipMaskPayload {
    fn default() -> Self {
        Self {
            kind: 1.0,
            opacity: 1.0,
            radii: Vec4::ZERO,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct BackdropPayload {
    params0: Vec4,
    params1: Vec4,
    tint: Vec4,
    uv_rect: Vec4,
}

impl Default for BackdropPayload {
    fn default() -> Self {
        Self {
            params0: Vec4::new(0.0, 0.0, 1.0, 1.0),
            params1: Vec4::new(1.0, 0.0, BACKDROP_DEBUG_FINAL, BACKDROP_QUALITY_MEDIUM),
            tint: Vec4::new(1.0, 1.0, 1.0, 1.0),
            uv_rect: Vec4::new(0.0, 0.0, 1.0, 1.0),
        }
    }
}

fn encode_backdrop(
    backdrop: Option<Backdrop>,
    scale_factor: f32,
    sample_region: BackdropSampleRegion,
    backdrop_enabled: bool,
    debug_view: BackdropDebugView,
) -> BackdropPayload {
    let mut payload = BackdropPayload::default();
    payload.params1.z = encode_backdrop_debug(debug_view);
    payload.uv_rect = sanitize_uv_rect(sample_region);

    let Some(backdrop) = backdrop else {
        return payload;
    };

    let sanitized = backdrop.sanitized();
    let enabled = backdrop_enabled && sanitized.is_active();

    payload.params0.x = if enabled { 1.0 } else { 0.0 };
    payload.params0.y = sanitized.blur * scale_factor;
    payload.params0.z = sanitized.brightness;
    payload.params0.w = sanitized.saturation;

    payload.params1.x = sanitized.contrast;
    payload.params1.y = sanitized.tint_opacity;
    payload.params1.w = encode_backdrop_quality(sanitized.quality);

    payload.tint = sanitize_linear_color(sanitized.tint.to_linear().to_vec4());
    payload
}

fn encode_backdrop_quality(quality: BackdropQuality) -> f32 {
    match quality {
        BackdropQuality::Low => BACKDROP_QUALITY_LOW,
        BackdropQuality::Medium => BACKDROP_QUALITY_MEDIUM,
        BackdropQuality::High => BACKDROP_QUALITY_HIGH,
    }
}

fn encode_backdrop_debug(mode: BackdropDebugView) -> f32 {
    match mode {
        BackdropDebugView::Final => BACKDROP_DEBUG_FINAL,
        BackdropDebugView::Source => BACKDROP_DEBUG_SOURCE,
        BackdropDebugView::Blurred => BACKDROP_DEBUG_BLURRED,
        BackdropDebugView::Mask => BACKDROP_DEBUG_MASK,
    }
}

fn sanitize_uv_rect(region: BackdropSampleRegion) -> Vec4 {
    let min = Vec2::new(
        sanitize_uv(region.min_uv.x, 0.0),
        sanitize_uv(region.min_uv.y, 0.0),
    );
    let max = Vec2::new(
        sanitize_uv(region.max_uv.x, 1.0),
        sanitize_uv(region.max_uv.y, 1.0),
    );

    let min = min.clamp(Vec2::ZERO, Vec2::ONE);
    let max = max.clamp(Vec2::ZERO, Vec2::ONE);
    let sorted_min = Vec2::new(min.x.min(max.x), min.y.min(max.y));
    let sorted_max = Vec2::new(max.x.max(min.x), max.y.max(min.y));

    Vec4::new(sorted_min.x, sorted_min.y, sorted_max.x, sorted_max.y)
}

fn encode_clip(clip: super::mask::Clip, logical_size: Vec2, scale_factor: f32) -> ClipMaskPayload {
    let clip = clip.sanitized();
    match clip.shape {
        Shape::RoundedRect(RoundedRect { radii }) => {
            let normalized = normalize_corner_radii(radii, logical_size);
            ClipMaskPayload {
                kind: 0.0,
                opacity: clip.opacity,
                radii: normalized.as_vec4() * scale_factor,
            }
        }
    }
}

fn encode_mask(mask: super::mask::Mask, logical_size: Vec2, scale_factor: f32) -> ClipMaskPayload {
    let mask = mask.sanitized();
    match mask.shape {
        Shape::RoundedRect(RoundedRect { radii }) => {
            let normalized = normalize_corner_radii(radii, logical_size);
            ClipMaskPayload {
                kind: 0.0,
                opacity: mask.opacity,
                radii: normalized.as_vec4() * scale_factor,
            }
        }
    }
}

fn encode_effects(
    effects: &Effects,
    scale_factor: f32,
) -> (OuterShadowUniform, OuterGlowUniform, InnerShadowUniform, Vec4) {
    let shadow = effects
        .outer_shadow
        .map(|value| encode_outer_shadow(value.sanitized(), scale_factor))
        .unwrap_or_default();
    let glow = effects
        .outer_glow
        .map(|value| encode_outer_glow(value.sanitized(), scale_factor))
        .unwrap_or_default();
    let inner = effects
        .inner_shadow
        .map(|value| encode_inner_shadow(value.sanitized(), scale_factor))
        .unwrap_or_default();

    let bounds = compute_effect_bounds(effects, scale_factor);
    (shadow, glow, inner, bounds)
}

fn encode_outer_shadow(shadow: OuterShadow, scale_factor: f32) -> OuterShadowUniform {
    OuterShadowUniform {
        color: sanitize_linear_color(shadow.color.to_linear().to_vec4()),
        offset_blur_spread: Vec4::new(
            shadow.offset.x * scale_factor,
            shadow.offset.y * scale_factor,
            shadow.blur * scale_factor,
            shadow.spread * scale_factor,
        ),
        opacity_and_falloff: Vec4::new(shadow.opacity, encode_shadow_falloff(shadow.falloff), 0.0, 0.0),
    }
}

fn encode_outer_glow(glow: OuterGlow, scale_factor: f32) -> OuterGlowUniform {
    OuterGlowUniform {
        color: sanitize_linear_color(glow.color.to_linear().to_vec4()),
        blur_spread_opacity_falloff: Vec4::new(
            glow.blur * scale_factor,
            glow.spread * scale_factor,
            glow.opacity,
            encode_shadow_falloff(glow.falloff),
        ),
    }
}

fn encode_inner_shadow(shadow: InnerShadow, scale_factor: f32) -> InnerShadowUniform {
    InnerShadowUniform {
        color: sanitize_linear_color(shadow.color.to_linear().to_vec4()),
        offset_blur_spread: Vec4::new(
            shadow.offset.x * scale_factor,
            shadow.offset.y * scale_factor,
            shadow.blur * scale_factor,
            shadow.spread * scale_factor,
        ),
        opacity_and_falloff: Vec4::new(shadow.opacity, encode_shadow_falloff(shadow.falloff), 0.0, 0.0),
    }
}

fn compute_effect_bounds(effects: &Effects, scale_factor: f32) -> Vec4 {
    let mut left: f32 = 0.0;
    let mut top: f32 = 0.0;
    let mut right: f32 = 0.0;
    let mut bottom: f32 = 0.0;

    if let Some(shadow) = effects.outer_shadow {
        let shadow = shadow.sanitized();
        let alpha = sanitize_linear_color(shadow.color.to_linear().to_vec4()).w;
        if shadow.opacity > 0.0 && alpha > 0.0 {
            let blur_extent = shadow_falloff_extent(shadow.blur, shadow.falloff);
            let base = (shadow.spread.max(0.0) + blur_extent + 1.0) * scale_factor;
            let dx = shadow.offset.x * scale_factor;
            let dy = shadow.offset.y * scale_factor;
            left = left.max((base - dx).max(0.0));
            right = right.max((base + dx).max(0.0));
            top = top.max((base - dy).max(0.0));
            bottom = bottom.max((base + dy).max(0.0));
        }
    }

    if let Some(glow) = effects.outer_glow {
        let glow = glow.sanitized();
        let alpha = sanitize_linear_color(glow.color.to_linear().to_vec4()).w;
        if glow.opacity > 0.0 && alpha > 0.0 {
            let blur_extent = shadow_falloff_extent(glow.blur, glow.falloff);
            let base = (glow.spread.max(0.0) + blur_extent + 1.0) * scale_factor;
            left = left.max(base);
            right = right.max(base);
            top = top.max(base);
            bottom = bottom.max(base);
        }
    }

    Vec4::new(left, top, right, bottom).min(Vec4::splat(MAX_RENDER_EXTENT))
}

fn shadow_falloff_extent(blur: f32, falloff: ShadowFalloff) -> f32 {
    let blur = blur.max(0.0);
    match falloff {
        ShadowFalloff::Linear => blur,
        ShadowFalloff::Smooth => blur * 1.25,
        ShadowFalloff::Gaussian => blur * 3.0,
    }
}

fn encode_shadow_falloff(falloff: ShadowFalloff) -> f32 {
    match falloff {
        ShadowFalloff::Linear => SHADOW_FALLOFF_LINEAR,
        ShadowFalloff::Smooth => SHADOW_FALLOFF_SMOOTH,
        ShadowFalloff::Gaussian => SHADOW_FALLOFF_GAUSSIAN,
    }
}

fn encode_paint(paint: &Paint) -> PaintUniform {
    match paint {
        Paint::Solid(color) => PaintUniform {
            kind_and_flags: Vec4::new(PAINT_KIND_SOLID, 0.0, 0.0, 0.0),
            solid_color: sanitize_linear_color(color.to_linear().to_vec4()),
            ..PaintUniform::default()
        },
        Paint::LinearGradient(gradient) => encode_linear_gradient(gradient),
        Paint::RadialGradient(gradient) => encode_radial_gradient(gradient),
        Paint::AngularGradient(gradient) => encode_angular_gradient(gradient),
        Paint::Shimmer(shimmer) => {
            let shimmer = shimmer.sanitized();
            // Tagged union: reuse unused gradient lanes; no uniform ABI growth.
            PaintUniform {
                kind_and_flags: Vec4::new(4.0, 0.0, 0.0, if shimmer.enabled { 1.0 } else { 0.0 }),
                solid_color: sanitize_linear_color(shimmer.base_color.to_linear().to_vec4()),
                linear_points: Vec4::new(shimmer.duration,
                    if shimmer.direction == super::ShimmerDirection::RightToLeft { -1.0 } else { 1.0 },
                    shimmer.phase, shimmer.width),
                radial_center_radius: Vec4::new(shimmer.softness, shimmer.intensity, 0.0, 0.0),
                angular_center_angle: sanitize_linear_color(shimmer.highlight_color.to_linear().to_vec4()),
                ..default()
            }
        }
    }
}

fn encode_linear_gradient(gradient: &LinearGradient) -> PaintUniform {
    let (stops, stop_count) = encode_stops(&gradient.stops);
    PaintUniform {
        kind_and_flags: Vec4::new(
            PAINT_KIND_LINEAR,
            stop_count,
            if gradient.dithering {
                PAINT_FLAG_DITHERING
            } else {
                0.0
            },
            0.0,
        ),
        linear_points: Vec4::new(
            sanitize_uv(gradient.start.x, 0.0),
            sanitize_uv(gradient.start.y, 0.5),
            sanitize_uv(gradient.end.x, 1.0),
            sanitize_uv(gradient.end.y, 0.5),
        ),
        stops,
        ..PaintUniform::default()
    }
}

fn encode_radial_gradient(gradient: &RadialGradient) -> PaintUniform {
    let (stops, stop_count) = encode_stops(&gradient.stops);
    PaintUniform {
        kind_and_flags: Vec4::new(
            PAINT_KIND_RADIAL,
            stop_count,
            if gradient.dithering {
                PAINT_FLAG_DITHERING
            } else {
                0.0
            },
            0.0,
        ),
        radial_center_radius: Vec4::new(
            sanitize_uv(gradient.center.x, 0.5),
            sanitize_uv(gradient.center.y, 0.5),
            sanitize_radius(gradient.radius.x, 0.5),
            sanitize_radius(gradient.radius.y, 0.5),
        ),
        stops,
        ..PaintUniform::default()
    }
}

fn encode_angular_gradient(gradient: &AngularGradient) -> PaintUniform {
    let (stops, stop_count) = encode_stops(&gradient.stops);
    PaintUniform {
        kind_and_flags: Vec4::new(
            PAINT_KIND_ANGULAR,
            stop_count,
            if gradient.dithering {
                PAINT_FLAG_DITHERING
            } else {
                0.0
            },
            0.0,
        ),
        angular_center_angle: Vec4::new(
            sanitize_uv(gradient.center.x, 0.5),
            sanitize_uv(gradient.center.y, 0.5),
            sanitize_angle(gradient.angle_radians),
            0.0,
        ),
        stops,
        ..PaintUniform::default()
    }
}

fn encode_stops(stops: &[GradientStop]) -> ([GradientStopUniform; MAX_GRADIENT_STOPS], f32) {
    let normalized = normalize_stops(stops);
    let mut gpu_stops = [GradientStopUniform::default(); MAX_GRADIENT_STOPS];

    let count = normalized.len().min(MAX_GRADIENT_STOPS);
    for (index, stop) in normalized.into_iter().take(count).enumerate() {
        gpu_stops[index] = GradientStopUniform {
            color: sanitize_linear_color(stop.color.to_linear().to_vec4()),
            position_and_pad: Vec4::new(stop.position, 0.0, 0.0, 0.0),
        };
    }

    (gpu_stops, count as f32)
}

fn sanitize_linear_color(color: Vec4) -> Vec4 {
    let mut out = color;
    if !out.x.is_finite() {
        out.x = 0.0;
    }
    if !out.y.is_finite() {
        out.y = 0.0;
    }
    if !out.z.is_finite() {
        out.z = 0.0;
    }
    if !out.w.is_finite() {
        out.w = 0.0;
    }

    Vec4::new(
        out.x.clamp(0.0, 1.0),
        out.y.clamp(0.0, 1.0),
        out.z.clamp(0.0, 1.0),
        out.w.clamp(0.0, 1.0),
    )
}

fn sanitize_uv(value: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value.clamp(-MAX_RENDER_EXTENT, MAX_RENDER_EXTENT)
    } else {
        fallback
    }
}

fn sanitize_radius(value: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value.clamp(1e-4, MAX_RENDER_EXTENT)
    } else {
        fallback
    }
}

fn sanitize_angle(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(-MAX_RENDER_EXTENT, MAX_RENDER_EXTENT)
    } else {
        0.0
    }
}

fn sanitize_finite(value: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        fallback
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_shape_uniform,
        build_shape_uniform_with_debug,
        build_shape_uniform_with_debug_and_time,
    };
    use crate::rendering::{
        AngularGradient,
        Backdrop,
        BackdropDebugView,
        BackdropQuality,
        Border,
        BorderWidths,
        Clip,
        CornerRadii,
        Effects,
        GradientStop,
        InnerShadow,
        LinearGradient,
        Mask,
        Noise,
        NoiseTarget,
        OuterGlow,
        OuterShadow,
        Paint,
        RadialGradient,
        ShadowFalloff,
        Shape,
        Surface,
        UiRenderDebugView,
    };
    use bevy::prelude::*;

    #[test]
    fn shimmer_kind4_packs_linear_straight_colors_and_all_control_lanes() {
        use crate::rendering::{Shimmer, ShimmerDirection};
        for (direction, sign) in [(ShimmerDirection::LeftToRight, 1.0),
            (ShimmerDirection::RightToLeft, -1.0)]
        {
            for enabled in [false, true] {
                let shimmer = Shimmer {
                    base_color: Color::srgba(0.5, 0.25, 0.75, 0.25),
                    highlight_color: Color::srgba(0.75, 0.5, 0.25, 0.75),
                    duration: 2.5, direction, phase: -0.25, width: 0.4,
                    softness: 0.5, intensity: 0.8, enabled,
                };
                let paint = super::encode_paint(&Paint::Shimmer(shimmer));
                assert_eq!(paint.kind_and_flags, Vec4::new(4.0, 0.0, 0.0, if enabled { 1.0 } else { 0.0 }));
                assert_eq!(paint.linear_points, Vec4::new(2.5, sign, 0.75, 0.4));
                assert_eq!(paint.radial_center_radius, Vec4::new(0.5, 0.8, 0.0, 0.0));
                assert_eq!(paint.solid_color, shimmer.base_color.to_linear().to_vec4());
                assert_eq!(paint.angular_center_angle, shimmer.highlight_color.to_linear().to_vec4());
                assert_eq!(paint.stops, [super::GradientStopUniform::default(); super::MAX_GRADIENT_STOPS]);
                // Distinguish straight-alpha storage from prematurely premultiplying.
                assert_ne!(paint.solid_color.x, paint.solid_color.x * paint.solid_color.w);
            }
        }
    }

    #[test]
    fn shimmer_kind4_sanitizes_direct_public_inputs_before_gpu_encoding() {
        use crate::rendering::Shimmer;
        for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, f32::MIN,
            -0.0, -f32::from_bits(1), f32::from_bits(1), 0.5, f32::MAX]
        {
            let shimmer = Shimmer {
                base_color: Color::linear_rgba(f32::NAN, -1.0, 2.0, f32::INFINITY),
                highlight_color: Color::linear_rgba(f32::NEG_INFINITY, 2.0, -1.0, 0.5),
                duration: value, phase: value, width: value,
                softness: value, intensity: value, ..default()
            };
            let paint = super::encode_paint(&Paint::Shimmer(shimmer));
            let sanitized = shimmer.sanitized();
            assert_eq!(paint.linear_points, Vec4::new(sanitized.duration, 1.0, sanitized.phase, sanitized.width));
            assert_eq!(paint.radial_center_radius, Vec4::new(sanitized.softness, sanitized.intensity, 0.0, 0.0));
            assert_eq!(paint.solid_color, Vec4::new(0.0, 0.0, 1.0, 0.0));
            assert_eq!(paint.angular_center_angle, Vec4::new(0.0, 1.0, 0.0, 0.5));
            for lane in [paint.kind_and_flags, paint.solid_color, paint.linear_points,
                paint.radial_center_radius, paint.angular_center_angle]
            {
                assert!(lane.is_finite(), "input={value}: {lane:?}");
            }
        }
    }

    #[test]
    fn shimmer_fill_and_border_payloads_are_independent_of_dpi_and_cpu_noise_time() {
        use crate::rendering::{Shimmer, ShimmerDirection};
        let fill = Paint::Shimmer(Shimmer::new(Color::BLACK, Color::WHITE));
        let border = Paint::Shimmer(Shimmer {
            direction: ShimmerDirection::RightToLeft, phase: 0.375, ..default()
        });
        let surface = Surface::rounded_rect_border(8.0, fill.clone(), 2.0, border.clone());
        for scale in [0.0, 1.0, 2.0, 64.0] {
            for time in [0.0, 1.5, 3599.0, f32::MAX] {
                let uniforms = build_shape_uniform_with_debug_and_time(&surface,
                    Vec2::new(120.0, 40.0), scale, default(), false,
                    BackdropDebugView::Final, UiRenderDebugView::Final, time, false);
                assert_eq!(uniforms.fill_paint, super::encode_paint(&fill));
                assert_eq!(uniforms.border_paint, super::encode_paint(&border));
            }
        }
    }

    #[test]
    fn shimmer_uses_canonical_radius_including_zero_size_and_huge_circles() {
        use crate::rendering::Shimmer;
        let paint = Paint::Shimmer(Shimmer::new(Color::BLACK, Color::WHITE));
        for size in [0.0, 48.0, super::MAX_RENDER_EXTENT, f32::MAX] {
            let surface = Surface::rounded_rect_fill(size * 0.5, paint.clone());
            for scale in [0.0, 1.0, 2.0, 64.0, f32::MAX, f32::INFINITY] {
                let uniforms = build_shape_uniform(&surface, Vec2::splat(size), scale,
                    default(), false, BackdropDebugView::Final);
                assert_bounded_geometry_and_opacity(&uniforms);
                let bounded_scale = scale.min(64.0);
                let extent = size.min(super::MAX_RENDER_EXTENT / bounded_scale.max(1.0)) * bounded_scale;
                assert_eq!(uniforms.size_and_kind, Vec4::new(extent, extent, 0.0, 0.0));
                assert_eq!(uniforms.corner_radii, Vec4::splat(extent * 0.5));
                assert_eq!(uniforms.effect_bounds, Vec4::ZERO);
                assert_eq!(uniforms.fill_paint, super::encode_paint(&paint));
            }
        }
        // A rounded rectangle shares the same geometry path; paint never supplies
        // a second radius or stretches a circle after unequal dimensions are set.
        for (radius, expected) in [(0.0, 0.0), (12.0, 24.0), (f32::MAX, 60.0)] {
            let surface = Surface::rounded_rect_fill(radius, paint.clone());
            let uniforms = build_shape_uniform(&surface, Vec2::new(100.0, 60.0), 2.0,
                default(), false, BackdropDebugView::Final);
            assert_eq!(uniforms.corner_radii, Vec4::splat(expected));
            assert_eq!(uniforms.size_and_kind.z, 0.0);
            assert_eq!(uniforms.fill_paint, super::encode_paint(&paint));
        }
    }

    #[test]
    fn liquid_glass_scales_only_pixel_metrics_and_encodes_profiles() {
        use crate::rendering::{GlassProfile, LiquidGlass};
        for (profile, code) in [(GlassProfile::Convex, 0.0), (GlassProfile::Squircle, 1.0),
            (GlassProfile::Concave, 2.0), (GlassProfile::Lip, 3.0)]
        {
            for scale in [1.0, 1.5, 2.0] {
                let glass = LiquidGlass {
                    thickness: 3.0, bezel_width: 4.0, specular_width: 0.75,
                    refractive_index: 1.5, specular_intensity: 0.25, fresnel: 0.125,
                    chromatic_aberration: 0.03125, press_amount: 0.5,
                    light_direction: Vec2::new(0.0, -2.0), profile, ..default()
                };
                let surface = Surface::rounded_rect_fill(8.0, Color::NONE)
                    .with_backdrop(Backdrop { liquid_glass: Some(glass), ..default() });
                let uniforms = build_shape_uniform(&surface, Vec2::new(100.0, 50.0), scale,
                    default(), true, BackdropDebugView::Final);
                assert_eq!(uniforms.glass_optics, Vec4::new(1.0, 3.0 * scale, 4.0 * scale, 1.5));
                assert_eq!(uniforms.glass_light, Vec4::new(0.25, 0.75 * scale, 0.125, 0.03125));
                assert_eq!(uniforms.glass_state, Vec4::new(code, 0.5, 0.0, -1.0));
                assert_eq!(uniforms.surface_axes, Vec4::new(1.0, 0.0, 0.0, 1.0));
                assert_eq!(uniforms.glass_debug, Vec4::ZERO);
            }
        }
    }

    #[test]
    fn liquid_glass_enable_gates_preserve_index_one_and_zero_thickness() {
        use crate::rendering::LiquidGlass;
        // IOR=1 and thickness=0 are valid optical no-displacement inputs, not
        // disable switches: the shader may still render specular/Fresnel light.
        for (glass, capture, reduced, expected) in [
            (Some(LiquidGlass::default()), true, false, 1.0),
            (None, true, false, 0.0),
            (Some(LiquidGlass { enabled: false, ..default() }), true, false, 0.0),
            (Some(LiquidGlass::default()), false, false, 0.0),
            (Some(LiquidGlass::default()), true, true, 0.0),
            (Some(LiquidGlass { refractive_index: 1.0, ..default() }), true, false, 1.0),
            (Some(LiquidGlass { thickness: 0.0, ..default() }), true, false, 1.0),
        ] {
            let surface = Surface::rounded_rect_fill(8.0, Color::NONE)
                .with_backdrop(Backdrop { liquid_glass: glass, ..default() });
            let uniforms = build_shape_uniform_with_debug_and_time(&surface,
                Vec2::splat(100.0), 2.0, default(), capture, BackdropDebugView::Final,
                UiRenderDebugView::Final, 0.0, reduced);
            assert_eq!(uniforms.glass_optics.x, expected, "{glass:?}, capture={capture}, reduced={reduced}");
            if expected == 0.0 {
                assert_eq!(uniforms.glass_optics, Vec4::ZERO);
            } else {
                let glass = glass.unwrap();
                assert_eq!(uniforms.glass_optics.y, glass.thickness * 2.0);
                assert_eq!(uniforms.glass_optics.w, glass.refractive_index);
            }
        }
    }

    #[test]
    fn liquid_glass_uniform_shader_type_size_matches_naga_layout() {
        use bevy::{render::render_resource::{DownlevelFlags, ShaderType, WgpuFeatures},
            shader::{Shader, ShaderCache, ShaderCacheSource}};
        // Parse the real declarations, including nested paints and scalar
        // padding, without external Bevy imports or a new Naga dependency.
        let shader = include_str!("shaders/ui_shape.wgsl");
        let declarations = shader.split_once("const MAX_GRADIENT_STOPS").unwrap().1
            .split_once("@group(1)").unwrap().0;
        let source = format!("const MAX_GRADIENT_STOPS{declarations}\n\
            @group(0) @binding(0) var<uniform> uniforms: UiShapeUniforms;");
        let mut cache = ShaderCache::new((), WgpuFeatures::empty(), DownlevelFlags::empty(),
            |_, source, _| {
                let ShaderCacheSource::Naga(module) = source else { panic!("expected Naga layout"); };
                let ty = module.types.iter().find(|(_, ty)|
                    ty.name.as_deref() == Some("UiShapeUniforms")).unwrap().1;
                Ok(ty.inner.size(module.to_ctx()))
            });
        let mut assets = Assets::<Shader>::default();
        let shader = Shader::from_wgsl(source, "glass_uniform_layout.wgsl");
        let handle = assets.add(shader.clone());
        cache.set_shader(handle.id(), shader);
        let size = cache.get(0, handle.id(), &[]).expect("uniform declarations must validate");
        assert_eq!(u64::from(*size), super::UiShapeUniform::min_size().get());
    }

    fn assert_bounded_geometry_and_opacity(uniforms: &super::UiShapeUniform) {
        let check = |value: Vec4| {
            assert!(value.is_finite(), "non-finite geometry: {value:?}");
            assert!(value.abs().cmple(Vec4::splat(super::MAX_RENDER_EXTENT)).all(),
                "unbounded geometry: {value:?}");
        };
        // Deliberately exclude Color payloads: these regressions exercise the
        // geometry/standalone-opacity boundary, not arbitrary color conversions.
        for value in [
            uniforms.size_and_kind, uniforms.corner_radii, uniforms.border_widths,
            uniforms.effect_bounds, uniforms.outer_shadow.offset_blur_spread,
            uniforms.outer_shadow.opacity_and_falloff,
            uniforms.outer_glow.blur_spread_opacity_falloff,
            uniforms.inner_shadow.offset_blur_spread,
            uniforms.inner_shadow.opacity_and_falloff,
            uniforms.focus_primary_metrics, uniforms.focus_secondary_metrics,
            uniforms.focus_flags, uniforms.focus_glow.blur_spread_opacity_falloff,
            uniforms.clip_radii, uniforms.mask_radii,
            uniforms.backdrop_params0, uniforms.backdrop_params1, uniforms.backdrop_uv_rect,
            uniforms.noise.params0, uniforms.noise.params1, uniforms.noise.params2,
            Vec4::new(uniforms.clip_kind, uniforms.mask_kind, uniforms.debug_view, 0.0),
        ] {
            check(value);
        }
        for paint in [
            &uniforms.fill_paint, &uniforms.border_paint,
            &uniforms.focus_primary_paint, &uniforms.focus_secondary_paint,
        ] {
            for value in [paint.kind_and_flags, paint.linear_points,
                paint.radial_center_radius, paint.angular_center_angle] {
                check(value);
            }
            for stop in paint.stops {
                check(stop.position_and_pad);
                assert!((0.0..=1.0).contains(&stop.position_and_pad.x));
            }
        }
        for opacity in [
            uniforms.clip_opacity, uniforms.mask_opacity,
            uniforms.outer_shadow.opacity_and_falloff.x,
            uniforms.inner_shadow.opacity_and_falloff.x,
            uniforms.outer_glow.blur_spread_opacity_falloff.z,
            uniforms.focus_primary_metrics.z, uniforms.focus_secondary_metrics.z,
            uniforms.focus_glow.blur_spread_opacity_falloff.z,
            uniforms.backdrop_params1.y, uniforms.noise.params0.z,
        ] {
            assert!((0.0..=1.0).contains(&opacity), "invalid opacity: {opacity}");
        }
        let width = uniforms.size_and_kind.x;
        let height = uniforms.size_and_kind.y;
        let le = |sum: f32, limit: f32| sum <= limit + limit * 1e-6 + 1e-4;
        for radii in [uniforms.corner_radii, uniforms.clip_radii, uniforms.mask_radii] {
            assert!(radii.cmpge(Vec4::ZERO).all());
            assert!(le(radii.x + radii.y, width));
            assert!(le(radii.w + radii.z, width));
            assert!(le(radii.x + radii.w, height));
            assert!(le(radii.y + radii.z, height));
        }
        assert!(uniforms.border_widths.cmpge(Vec4::ZERO).all());
        assert!(le(uniforms.border_widths.y + uniforms.border_widths.w, width));
        assert!(le(uniforms.border_widths.x + uniforms.border_widths.z, height));
    }

    #[test]
    fn gpu_uniform_bounds_raw_geometry_and_opacity_for_all_float_classes() {
        let values = [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, f32::MAX,
            f32::MIN, 0.0, f32::from_bits(1), 0.5, 2.0, 64.0];
        for value in values {
            let shape = Shape::rounded_rect_corners(value, 8.0, value, 0.0);
            let stops = vec![GradientStop::new(value, Color::WHITE)];
            let mut surface = Surface::new(shape, Paint::linear(LinearGradient {
                start: Vec2::new(value, -value), end: Vec2::new(-value, value),
                stops: stops.clone(), dithering: true,
            }));
            surface.border = Some(Border {
                width: BorderWidths::sides(value, 8.0, value, 0.0),
                paint: Paint::radial(RadialGradient {
                    center: Vec2::splat(value), radius: Vec2::new(value, -value),
                    stops: stops.clone(), dithering: true,
                }),
            });
            surface.clip = Some(Clip { shape, opacity: value });
            surface.mask = Some(Mask { shape, opacity: value });
            surface.effects = Effects {
                outer_shadow: Some(OuterShadow {
                    color: Color::BLACK, offset: Vec2::new(value, -value),
                    blur: value, spread: value, opacity: value,
                    falloff: ShadowFalloff::Gaussian,
                }),
                outer_glow: Some(OuterGlow {
                    color: Color::WHITE, blur: value, spread: value, opacity: value,
                    falloff: ShadowFalloff::Gaussian,
                }),
                inner_shadow: Some(InnerShadow {
                    color: Color::BLACK, offset: Vec2::new(-value, value),
                    blur: value, spread: -value, opacity: value,
                    falloff: ShadowFalloff::Smooth,
                }),
            };
            let layer = super::FocusRingLayer {
                width: value, offset: value, opacity: value,
                paint: Paint::angular(AngularGradient {
                    center: Vec2::splat(value), angle_radians: value,
                    stops, dithering: true,
                }),
            };
            surface.decorations.focus_ring = Some(super::FocusRing {
                primary: layer.clone(), secondary: Some(layer),
                placement: super::FocusRingPlacement::Outside,
                glow: surface.effects.outer_glow,
            });
            surface.backdrop = Some(Backdrop {
                blur: value, tint_opacity: value, brightness: value,
                saturation: value, contrast: value, ..Backdrop::default()
            });
            surface.noise = Some(Noise {
                scale: value, strength: value, seed: value, speed: value,
                animated: true, ..Noise::default()
            });
            for scale in values {
                for size in [Vec2::new(100.0, 60.0), Vec2::ZERO,
                    Vec2::new(value, 60.0), Vec2::new(100.0, value),
                    Vec2::splat(value), Vec2::splat(f32::MAX)] {
                    let uniforms = build_shape_uniform_with_debug_and_time(
                        &surface, size, scale,
                        super::BackdropSampleRegion {
                            min_uv: Vec2::new(value, -value),
                            max_uv: Vec2::new(-value, value),
                        },
                        true, BackdropDebugView::Final, UiRenderDebugView::Final,
                        value, false,
                    );
                    assert_bounded_geometry_and_opacity(&uniforms);
                }
            }
        }
    }

    #[test]
    fn gpu_uniform_direct_clip_mask_opacity_matches_existing_sanitizers() {
        for (input, expected) in [
            (f32::NAN, 1.0), (f32::INFINITY, 1.0), (f32::NEG_INFINITY, 1.0),
            (f32::MAX, 1.0), (f32::MIN, 0.0), (-0.5, 0.0), (0.0, 0.0),
            (0.375, 0.375), (1.0, 1.0), (2.0, 1.0),
        ] {
            let mut surface = Surface::rounded_rect_fill(8.0, Color::WHITE);
            surface.clip = Some(Clip { shape: surface.shape, opacity: input });
            surface.mask = Some(Mask { shape: surface.shape, opacity: input });
            let uniforms = build_shape_uniform(
                &surface, Vec2::new(100.0, 60.0), 2.0,
                super::BackdropSampleRegion::default(), true, BackdropDebugView::Final,
            );
            assert_eq!(uniforms.clip_opacity, expected);
            assert_eq!(uniforms.mask_opacity, expected);
        }
    }

    #[test]
    fn gpu_uniform_preserves_valid_asymmetry_and_out_of_unit_gradient_coordinates() {
        let shape = Shape::rounded_rect_corners(80.0, 10.0, 5.0, 10.0);
        let surface = Surface::new(shape, Paint::linear(LinearGradient::new(
            Vec2::new(-2.0, 0.5), Vec2::new(3.0, 0.5), vec![],
        )))
        .border(Border::per_side(BorderWidths::sides(80.0, 10.0, 5.0, 70.0),
            Paint::radial(RadialGradient::circular(Vec2::new(-1.0, 2.0), 3.0, vec![]))))
        .with_clip(Clip::new(shape)).with_mask(Mask::new(shape));
        let uniforms = build_shape_uniform(
            &surface, Vec2::splat(100.0), 2.0,
            super::BackdropSampleRegion::default(), true, BackdropDebugView::Final,
        );
        assert_eq!(uniforms.corner_radii, Vec4::new(160.0, 20.0, 10.0, 20.0));
        assert_eq!(uniforms.clip_radii, uniforms.corner_radii);
        assert_eq!(uniforms.mask_radii, uniforms.corner_radii);
        assert_eq!(uniforms.border_widths, Vec4::new(160.0, 20.0, 10.0, 140.0));
        assert_eq!(uniforms.fill_paint.linear_points, Vec4::new(-2.0, 0.5, 3.0, 0.5));
        assert_eq!(uniforms.border_paint.radial_center_radius, Vec4::new(-1.0, 2.0, 3.0, 3.0));
    }

    #[test]
    fn gpu_uniform_converts_to_physical_units() {
        let surface = Surface::new(
            Shape::RoundedRect(crate::rendering::RoundedRect {
                radii: CornerRadii::new(6.0),
            }),
            Paint::solid(Color::srgb(1.0, 0.5, 0.25)),
        );

        let uniforms = build_shape_uniform(
            &surface,
            Vec2::new(100.0, 50.0),
            2.0,
            super::BackdropSampleRegion::default(),
            true,
            BackdropDebugView::Final,
        );
        assert_eq!(uniforms.size_and_kind.x, 200.0);
        assert_eq!(uniforms.size_and_kind.y, 100.0);
        assert_eq!(uniforms.corner_radii, Vec4::splat(12.0));
        assert_eq!(uniforms.border_widths, Vec4::ZERO);
        assert_eq!(uniforms.fill_paint.kind_and_flags.x, 0.0);
    }

    #[test]
    fn gpu_uniform_normalizes_invalid_radii() {
        let surface = Surface::new(
            Shape::rounded_rect_corners(100.0, 100.0, 10.0, 10.0),
            Paint::solid(Color::WHITE),
        );

        let uniforms = build_shape_uniform(
            &surface,
            Vec2::new(100.0, 40.0),
            1.0,
            super::BackdropSampleRegion::default(),
            true,
            BackdropDebugView::Final,
        );
        assert!(uniforms.corner_radii.x + uniforms.corner_radii.y <= 100.0001);
    }

    #[test]
    fn gpu_uniform_converts_border_widths_and_color() {
        let surface = Surface::new(Shape::rounded_rect(12.0), Paint::solid(Color::srgb(0.2, 0.3, 0.4)))
            .border(Border::per_side(
                BorderWidths::sides(1.0, 2.0, 3.0, 4.0),
                Paint::solid(Color::srgb(0.9, 0.1, 0.2)),
            ));

        let uniforms = build_shape_uniform(
            &surface,
            Vec2::new(100.0, 50.0),
            2.0,
            super::BackdropSampleRegion::default(),
            true,
            BackdropDebugView::Final,
        );
        assert_eq!(uniforms.border_widths, Vec4::new(2.0, 4.0, 6.0, 8.0));
        assert!(uniforms.border_paint.solid_color.w > 0.0);
    }

    #[test]
    fn gpu_uniform_normalizes_oversized_border_widths() {
        let surface = Surface::new(Shape::rounded_rect(8.0), Paint::solid(Color::WHITE)).border(
            Border::per_side(
                BorderWidths::sides(10.0, 100.0, 10.0, 100.0),
                Paint::solid(Color::BLACK),
            ),
        );

        let uniforms = build_shape_uniform(
            &surface,
            Vec2::new(80.0, 40.0),
            1.0,
            super::BackdropSampleRegion::default(),
            true,
            BackdropDebugView::Final,
        );
        let left = uniforms.border_widths.w;
        let right = uniforms.border_widths.y;
        assert!(left + right <= 80.0001);
    }

    #[test]
    fn gpu_uniform_encodes_linear_gradient_and_clamps_stop_count() {
        let gradient = LinearGradient::horizontal(vec![
            GradientStop::new(0.0, Color::srgb(1.0, 0.0, 0.0)),
            GradientStop::new(0.25, Color::srgb(1.0, 1.0, 0.0)),
            GradientStop::new(0.5, Color::srgb(0.0, 1.0, 0.0)),
            GradientStop::new(0.75, Color::srgb(0.0, 1.0, 1.0)),
            GradientStop::new(1.0, Color::srgb(0.0, 0.0, 1.0)),
        ]);
        let surface = Surface::new(Shape::rounded_rect(8.0), Paint::linear(gradient));

        let uniforms = build_shape_uniform(
            &surface,
            Vec2::new(120.0, 40.0),
            1.0,
            super::BackdropSampleRegion::default(),
            true,
            BackdropDebugView::Final,
        );
        assert_eq!(uniforms.fill_paint.kind_and_flags.x, 1.0);
        assert_eq!(uniforms.fill_paint.kind_and_flags.y, 4.0);
        assert_eq!(uniforms.fill_paint.kind_and_flags.z, 1.0);
    }

    #[test]
    fn gpu_uniform_encodes_effect_payloads_and_bounds() {
        let effects = Effects::default()
            .with_outer_shadow(
                OuterShadow::new(Color::srgba(0.0, 0.0, 0.0, 1.0))
                    .with_offset(Vec2::new(4.0, 6.0))
                    .with_blur(10.0)
                    .with_spread(2.0)
                    .with_opacity(0.25)
                    .with_falloff(ShadowFalloff::Gaussian),
            )
            .with_inner_shadow(
                InnerShadow::new(Color::srgba(0.0, 0.0, 0.0, 1.0))
                    .with_offset(Vec2::new(-2.0, 3.0))
                    .with_blur(6.0)
                    .with_spread(2.0)
                    .with_opacity(0.4)
                    .with_falloff(ShadowFalloff::Smooth),
            )
            .with_outer_glow(
                OuterGlow::new(Color::srgba(0.3, 0.6, 1.0, 1.0))
                    .with_blur(8.0)
                    .with_spread(1.0)
                    .with_opacity(0.3)
                    .with_falloff(ShadowFalloff::Smooth),
            );

        let surface = Surface::rounded_rect_fill(10.0, Color::WHITE).with_effects(effects);
        let uniforms = build_shape_uniform(
            &surface,
            Vec2::new(120.0, 40.0),
            2.0,
            super::BackdropSampleRegion::default(),
            true,
            BackdropDebugView::Final,
        );

        assert_eq!(uniforms.outer_shadow.offset_blur_spread.x, 8.0);
        assert_eq!(uniforms.outer_shadow.offset_blur_spread.y, 12.0);
        assert_eq!(uniforms.outer_shadow.offset_blur_spread.z, 20.0);
        assert_eq!(uniforms.outer_shadow.offset_blur_spread.w, 4.0);
        assert_eq!(uniforms.inner_shadow.offset_blur_spread.x, -4.0);
        assert_eq!(uniforms.inner_shadow.offset_blur_spread.y, 6.0);
        assert_eq!(uniforms.inner_shadow.opacity_and_falloff.x, 0.4);
        assert!(uniforms.effect_bounds.x > 0.0);
        assert!(uniforms.effect_bounds.y > 0.0);
        assert!(uniforms.effect_bounds.z > uniforms.effect_bounds.x);
        assert!(uniforms.effect_bounds.w > uniforms.effect_bounds.y);
    }

    #[test]
    fn gpu_uniform_sanitizes_invalid_effect_inputs() {
        let effects = Effects::default().with_outer_shadow(
            OuterShadow::new(Color::srgba(0.0, 0.0, 0.0, 1.0))
                .with_offset(Vec2::new(f32::NAN, f32::INFINITY))
                .with_blur(f32::NEG_INFINITY)
                .with_spread(f32::NAN)
                .with_opacity(f32::INFINITY),
        );

        let surface = Surface::rounded_rect_fill(8.0, Color::WHITE).with_effects(effects);
        let uniforms = build_shape_uniform(
            &surface,
            Vec2::new(80.0, 40.0),
            1.0,
            super::BackdropSampleRegion::default(),
            true,
            BackdropDebugView::Final,
        );

        assert_eq!(uniforms.outer_shadow.offset_blur_spread.x, 0.0);
        assert!(uniforms.outer_shadow.offset_blur_spread.y.is_finite());
        assert_eq!(uniforms.outer_shadow.offset_blur_spread.z, 0.0);
        assert_eq!(uniforms.outer_shadow.opacity_and_falloff.x, 1.0);
    }

    #[test]
    fn gpu_uniform_encodes_shape_clip_and_mask() {
        let surface = Surface::new(
            Shape::rounded_rect(12.0),
            Paint::solid(Color::WHITE),
        )
        .with_clip(Clip::rounded_rect(8.0).with_opacity(0.75))
        .with_mask(Mask::rounded_rect(10.0).with_opacity(0.5));

        let uniforms = build_shape_uniform(
            &surface,
            Vec2::new(100.0, 60.0),
            1.0,
            super::BackdropSampleRegion::default(),
            true,
            BackdropDebugView::Final,
        );
        assert_eq!(uniforms.clip_kind, 0.0);
        assert_eq!(uniforms.clip_opacity, 0.75);
        assert_eq!(uniforms.mask_kind, 0.0);
        assert_eq!(uniforms.mask_opacity, 0.5);
        assert_eq!(uniforms.clip_radii, Vec4::splat(8.0));
        assert_eq!(uniforms.mask_radii, Vec4::splat(10.0));
    }

    #[test]
    fn clip_and_mask_are_distinct_semantics() {
        let clip = Clip::rounded_rect(16.0);
        let mask = Mask::rounded_rect(16.0).with_opacity(0.4);

        assert_ne!(clip.opacity, mask.opacity);
        assert_eq!(clip.shape, Shape::rounded_rect(16.0));
        assert_eq!(mask.shape, Shape::rounded_rect(16.0));
    }

    #[test]
    fn gpu_uniform_encodes_backdrop_payload() {
        let surface = Surface::rounded_rect_fill(16.0, Color::srgba(1.0, 1.0, 1.0, 0.08))
            .with_backdrop(
                Backdrop::new()
                    .with_blur(16.0)
                    .with_tint(Color::srgba(0.9, 0.95, 1.0, 1.0))
                    .with_tint_opacity(0.1)
                    .with_brightness(0.95)
                    .with_saturation(0.85)
                    .with_contrast(1.1)
                    .with_quality(BackdropQuality::High),
            );

        let uniforms = build_shape_uniform(
            &surface,
            Vec2::new(200.0, 120.0),
            2.0,
            super::BackdropSampleRegion {
                min_uv: Vec2::new(0.2, 0.3),
                max_uv: Vec2::new(0.8, 0.9),
            },
            true,
            BackdropDebugView::Blurred,
        );

        assert_eq!(uniforms.backdrop_params0.x, 1.0);
        assert_eq!(uniforms.backdrop_params0.y, 32.0);
        assert_eq!(uniforms.backdrop_params0.z, 0.95);
        assert_eq!(uniforms.backdrop_params0.w, 0.85);
        assert_eq!(uniforms.backdrop_params1.x, 1.1);
        assert_eq!(uniforms.backdrop_params1.y, 0.1);
        assert_eq!(uniforms.backdrop_params1.z, 2.0);
        assert_eq!(uniforms.backdrop_params1.w, 2.0);
        assert_eq!(uniforms.backdrop_uv_rect, Vec4::new(0.2, 0.3, 0.8, 0.9));
    }

    #[test]
    fn gpu_uniform_encodes_gradient_border_paint() {
        let border_gradient = LinearGradient::angle_degrees(
            30.0,
            vec![
                GradientStop::new(0.0, Color::srgba(1.0, 0.0, 0.0, 1.0)),
                GradientStop::new(0.6, Color::srgba(0.0, 0.5, 1.0, 0.5)),
                GradientStop::new(1.0, Color::srgba(1.0, 0.0, 1.0, 0.0)),
            ],
        );

        let surface = Surface::rounded_rect_fill(14.0, Color::srgba(1.0, 1.0, 1.0, 0.08))
            .border(Border::new(3.0, Paint::linear(border_gradient)));

        let uniforms = build_shape_uniform(
            &surface,
            Vec2::new(180.0, 72.0),
            1.0,
            super::BackdropSampleRegion::default(),
            true,
            BackdropDebugView::Final,
        );

        assert_eq!(uniforms.border_paint.kind_and_flags.x, 1.0);
        assert_eq!(uniforms.border_paint.kind_and_flags.y, 3.0);
        assert_eq!(uniforms.border_widths, Vec4::splat(3.0));
        assert!((uniforms.border_paint.stops[2].color.w - 0.0).abs() < 1e-6);
    }

    #[test]
    fn gpu_uniform_sanitizes_unsorted_gradient_border_stops() {
        let border_gradient = RadialGradient::new(
            Vec2::new(0.5, 0.5),
            Vec2::new(0.6, 0.6),
            vec![
                GradientStop::new(1.5, Color::WHITE),
                GradientStop::new(0.5, Color::BLACK),
                GradientStop::new(0.5, Color::srgba(0.2, 0.4, 0.8, 0.6)),
                GradientStop::new(f32::NAN, Color::srgba(1.0, 0.0, 0.0, 0.5)),
            ],
        );

        let surface = Surface::rounded_rect_fill(10.0, Color::WHITE)
            .border(Border::new(2.0, Paint::radial(border_gradient)));

        let uniforms = build_shape_uniform(
            &surface,
            Vec2::new(160.0, 52.0),
            1.0,
            super::BackdropSampleRegion::default(),
            true,
            BackdropDebugView::Final,
        );

        assert_eq!(uniforms.border_paint.kind_and_flags.x, 2.0);
        assert_eq!(uniforms.border_paint.kind_and_flags.y, 3.0);

        let p0 = uniforms.border_paint.stops[0].position_and_pad.x;
        let p1 = uniforms.border_paint.stops[1].position_and_pad.x;
        let p2 = uniforms.border_paint.stops[2].position_and_pad.x;
        assert!(p0 <= p1 && p1 <= p2);
        assert_eq!(p0, 0.0);
        assert_eq!(p2, 1.0);
    }

    #[test]
    fn gpu_uniform_encodes_render_debug_view() {
        let surface = Surface::rounded_rect_fill(
            12.0,
            Paint::angular(AngularGradient::angle_degrees(
                Vec2::new(0.5, 0.5),
                90.0,
                vec![
                    GradientStop::new(0.0, Color::srgb(1.0, 0.0, 0.0)),
                    GradientStop::new(1.0, Color::srgb(0.0, 0.0, 1.0)),
                ],
            )),
        );

        let uniforms = build_shape_uniform_with_debug(
            &surface,
            Vec2::new(100.0, 50.0),
            1.0,
            super::BackdropSampleRegion::default(),
            true,
            BackdropDebugView::Final,
            UiRenderDebugView::GradientUv,
        );

        assert_eq!(uniforms.debug_view, 3.0);
    }

    #[test]
    fn gpu_uniform_encodes_noise_payload() {
        let surface = Surface::rounded_rect_fill(12.0, Color::WHITE).with_noise(
            Noise::grain(18.0, 0.03)
                .with_seed(42.0)
                .with_animated(true)
                .with_speed(0.7)
                .with_target(NoiseTarget::Border),
        );

        let uniforms = build_shape_uniform_with_debug_and_time(
            &surface,
            Vec2::new(100.0, 50.0),
            1.0,
            super::BackdropSampleRegion::default(),
            true,
            BackdropDebugView::Final,
            UiRenderDebugView::Final,
            12.5,
            false,
        );

        assert_eq!(uniforms.noise.params0.x, 1.0);
        assert_eq!(uniforms.noise.params0.z, 0.03);
        assert_eq!(uniforms.noise.params0.w, 18.0);
        assert_eq!(uniforms.noise.params1.x, 42.0);
        assert_eq!(uniforms.noise.params1.y, 0.7);
        assert_eq!(uniforms.noise.params1.z, 1.0);
        assert_eq!(uniforms.noise.params2.x, 12.5);
        assert_eq!(uniforms.noise.params2.y, 2.0);
    }

    #[test]
    fn gpu_uniform_noise_respects_reduced_effects() {
        let surface = Surface::rounded_rect_fill(12.0, Color::WHITE)
            .with_noise(Noise::grain(20.0, 0.08));

        let uniforms = build_shape_uniform_with_debug_and_time(
            &surface,
            Vec2::new(100.0, 50.0),
            1.0,
            super::BackdropSampleRegion::default(),
            true,
            BackdropDebugView::Final,
            UiRenderDebugView::Final,
            3.0,
            true,
        );

        assert_eq!(uniforms.noise.params0.x, 0.0);
        assert_eq!(uniforms.noise.params0.z, 0.0);
    }

    #[test]
    fn gpu_uniform_encodes_noise_debug_view() {
        let surface = Surface::rounded_rect_fill(12.0, Color::WHITE);

        let uniforms = build_shape_uniform_with_debug(
            &surface,
            Vec2::new(40.0, 24.0),
            1.0,
            super::BackdropSampleRegion::default(),
            true,
            BackdropDebugView::Final,
            UiRenderDebugView::NoiseRaw,
        );

        assert_eq!(uniforms.debug_view, 5.0);
    }
}
