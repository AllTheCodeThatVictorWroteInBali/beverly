#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum UiRenderDebugView {
    #[default]
    Final,
    Sdf,
    BorderCoverage,
    GradientUv,
    BorderPaint,
    NoiseRaw,
    NoiseCoords,
    NoiseStrength,
    NoiseModulation,
    FocusCoverage,
    FocusDistance,
}

impl UiRenderDebugView {
    pub fn from_env(value: Option<&str>) -> Self {
        match value {
            Some("sdf") | Some("distance") => Self::Sdf,
            Some("border") | Some("border_coverage") => Self::BorderCoverage,
            Some("uv") | Some("gradient_uv") => Self::GradientUv,
            Some("border_paint") | Some("paint") => Self::BorderPaint,
            Some("noise") | Some("noise_raw") => Self::NoiseRaw,
            Some("noise_coords") | Some("noise_uv") => Self::NoiseCoords,
            Some("noise_strength") => Self::NoiseStrength,
            Some("noise_mod") | Some("noise_modulation") => Self::NoiseModulation,
            Some("focus") | Some("focus_coverage") => Self::FocusCoverage,
            Some("focus_distance") => Self::FocusDistance,
            _ => Self::Final,
        }
    }
}