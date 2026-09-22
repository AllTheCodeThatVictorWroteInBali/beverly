#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

struct FullscreenEffect {
    intensity: f32,
};

@group(0) @binding(0)
var screen_texture: texture_2d<f32>;
@group(0) @binding(1)
var screen_sampler: sampler;

@group(0) @binding(2)
var<uniform> effect: FullscreenEffect;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let radius = effect.intensity * 0.012;
    var color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    var total = 0.0;

    let offsets = array<vec2<f32>, 9>(
        vec2<f32>(-1.0, -1.0), vec2<f32>(0.0, -1.0), vec2<f32>(1.0, -1.0),
        vec2<f32>(-1.0, 0.0),  vec2<f32>(0.0, 0.0),  vec2<f32>(1.0, 0.0),
        vec2<f32>(-1.0, 1.0),  vec2<f32>(0.0, 1.0),  vec2<f32>(1.0, 1.0)
    );
    let weights = array<f32, 9>(0.05, 0.09, 0.05, 0.09, 0.16, 0.09, 0.05, 0.09, 0.05);

    for (var i = 0; i < 9; i++) {
        let sample_uv = uv + offsets[i] * radius;
        color += textureSample(screen_texture, screen_sampler, sample_uv) * weights[i];
        total += weights[i];
    }

    color = color / total;
    return vec4<f32>(color.rgb, clamp(color.a + 0.18, 0.0, 1.0));
}
