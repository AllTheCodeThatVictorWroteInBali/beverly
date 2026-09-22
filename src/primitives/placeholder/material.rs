use bevy::{
    asset::Asset,
    color::LinearRgba,
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderType},
    shader::ShaderRef,
};

#[derive(Clone, Copy, Debug, ShaderType)]
pub struct PlaceholderUniforms {
    pub base_color: LinearRgba,
    pub highlight_color: LinearRgba,

    /// Animation time in seconds.
    pub time: f32,

    /// 0 = none
    /// 1 = shimmer
    /// 2 = pulse
    /// 3 = fade
    /// 4 = light reveal
    /// 5 = circular reveal
    pub effect: f32,

    /// Effect intensity.
    pub intensity: f32,

    /// Animation speed.
    pub speed: f32,

    /// Angle of the shimmer in radians.
    pub angle: f32,

    /// Width of the highlight band.
    pub width: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct PlaceholderMaterial {
    #[uniform(0)]
    pub uniforms: PlaceholderUniforms,
}

impl UiMaterial for PlaceholderMaterial {
    fn fragment_shader() -> ShaderRef {
        "embedded://beverly/primitives/placeholder/placeholder.wgsl".into()
    }
}
