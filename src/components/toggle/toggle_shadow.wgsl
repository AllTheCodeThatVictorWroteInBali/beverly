#import bevy_ui::ui_vertex_output::UiVertexOutput

struct ToggleShadowUniforms {
    color: vec4<f32>,
    radius: f32,
    softness: f32,
};

@group(1) @binding(0)
var<uniform> uniforms: ToggleShadowUniforms;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let centered = uv - vec2<f32>(0.5, 0.5);
    let distance = length(centered * vec2<f32>(1.8, 1.3));
    let spread = max(uniforms.radius, 0.05);
    let falloff = clamp(1.0 - (distance / spread), 0.0, 1.0);
    let softness = clamp(uniforms.softness, 0.0, 1.0);
    let alpha = pow(falloff, 1.5 + (1.0 - softness) * 2.0) * uniforms.color.a;
    return vec4<f32>(uniforms.color.rgb, alpha);
}
