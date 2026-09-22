#import bevy_ui::ui_vertex_output::UiVertexOutput

struct PlaceholderUniforms {
    base_color: vec4<f32>,
    highlight_color: vec4<f32>,
    time: f32,
    effect: f32,
    intensity: f32,
    speed: f32,
    angle: f32,
    width: f32,
};

@group(1) @binding(0)
var<uniform> uniforms: PlaceholderUniforms;

fn effect_shimmer(uv: vec2<f32>) -> f32 {
    let t = uniforms.time * uniforms.speed;
    let direction = vec2<f32>(cos(uniforms.angle), sin(uniforms.angle));
    let center = 0.5 + 0.5 * sin(t);
    let projection = dot(uv, direction);
    let half_width = max(uniforms.width * 0.5, 0.01);
    return 1.0 - smoothstep(center - half_width, center + half_width, projection);
}

fn effect_pulse() -> f32 {
    return 0.5 + 0.5 * sin(uniforms.time * uniforms.speed * 2.0);
}

fn effect_fade() -> f32 {
    return 0.3 + 0.7 * abs(sin(uniforms.time * uniforms.speed));
}

fn effect_light_reveal(uv: vec2<f32>) -> f32 {
    let progress = fract(uniforms.time * uniforms.speed * 0.25);
    return smoothstep(progress - uniforms.width, progress, uv.x);
}

fn effect_circular_reveal(uv: vec2<f32>) -> f32 {
    let progress = fract(uniforms.time * uniforms.speed * 0.25);
    let dist = distance(uv, vec2<f32>(0.5, 0.5));
    return 1.0 - smoothstep(progress - uniforms.width, progress, dist);
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;

    var mask = 0.0;
    if uniforms.effect < 0.5 {
        mask = 0.0;
    } else if uniforms.effect < 1.5 {
        mask = effect_shimmer(uv);
    } else if uniforms.effect < 2.5 {
        mask = effect_pulse();
    } else if uniforms.effect < 3.5 {
        mask = effect_fade();
    } else if uniforms.effect < 4.5 {
        mask = effect_light_reveal(uv);
    } else {
        mask = effect_circular_reveal(uv);
    }

    let blend = clamp(mask * uniforms.intensity, 0.0, 1.0);
    return mix(uniforms.base_color, uniforms.highlight_color, blend);
}
