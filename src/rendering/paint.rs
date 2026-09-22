use bevy::{color::LinearRgba, prelude::*};

pub const MAX_GRADIENT_STOPS: usize = 4;

#[derive(Clone, Debug, PartialEq)]
pub struct GradientStop {
    pub position: f32,
    pub color: Color,
}

impl GradientStop {
    pub fn new(position: f32, color: Color) -> Self {
        Self { position, color }
    }

    pub fn at(position: f32, color: Color) -> Self {
        Self::new(position, color)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LinearGradient {
    /// Normalized local coordinates where (0,0)=top-left and (1,1)=bottom-right.
    pub start: Vec2,
    /// Normalized local coordinates where (0,0)=top-left and (1,1)=bottom-right.
    pub end: Vec2,
    pub stops: Vec<GradientStop>,
    pub dithering: bool,
}

impl LinearGradient {
    pub fn new(start: Vec2, end: Vec2, stops: impl Into<Vec<GradientStop>>) -> Self {
        Self {
            start,
            end,
            stops: stops.into(),
            dithering: true,
        }
    }

    pub fn horizontal(stops: impl Into<Vec<GradientStop>>) -> Self {
        Self::new(Vec2::new(0.0, 0.5), Vec2::new(1.0, 0.5), stops)
    }

    pub fn vertical(stops: impl Into<Vec<GradientStop>>) -> Self {
        Self::new(Vec2::new(0.5, 0.0), Vec2::new(0.5, 1.0), stops)
    }

    pub fn angle_degrees(angle_degrees: f32, stops: impl Into<Vec<GradientStop>>) -> Self {
        Self::angle_radians(angle_degrees.to_radians(), stops)
    }

    pub fn angle_radians(angle_radians: f32, stops: impl Into<Vec<GradientStop>>) -> Self {
        let dir = Vec2::new(angle_radians.cos(), angle_radians.sin());
        let half = dir * 0.5;
        Self::new(Vec2::splat(0.5) - half, Vec2::splat(0.5) + half, stops)
    }

    pub fn with_dithering(mut self, enabled: bool) -> Self {
        self.dithering = enabled;
        self
    }

    pub fn normalized_stops(&self) -> Vec<GradientStop> {
        normalize_stops(&self.stops)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RadialGradient {
    /// Normalized local coordinates where (0,0)=top-left and (1,1)=bottom-right.
    pub center: Vec2,
    /// Normalized local radius where (1,1) reaches right/bottom edge from center.
    pub radius: Vec2,
    pub stops: Vec<GradientStop>,
    pub dithering: bool,
}

impl RadialGradient {
    pub fn new(center: Vec2, radius: Vec2, stops: impl Into<Vec<GradientStop>>) -> Self {
        Self {
            center,
            radius,
            stops: stops.into(),
            dithering: true,
        }
    }

    pub fn circular(center: Vec2, radius: f32, stops: impl Into<Vec<GradientStop>>) -> Self {
        Self::new(center, Vec2::splat(radius), stops)
    }

    pub fn with_dithering(mut self, enabled: bool) -> Self {
        self.dithering = enabled;
        self
    }

    pub fn normalized_stops(&self) -> Vec<GradientStop> {
        normalize_stops(&self.stops)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AngularGradient {
    /// Normalized local coordinates where (0,0)=top-left and (1,1)=bottom-right.
    pub center: Vec2,
    /// Zero direction points to +X (right). In UI space (y down), positive angles rotate clockwise.
    pub angle_radians: f32,
    pub stops: Vec<GradientStop>,
    pub dithering: bool,
}

impl AngularGradient {
    pub fn new(center: Vec2, angle_radians: f32, stops: impl Into<Vec<GradientStop>>) -> Self {
        Self {
            center,
            angle_radians,
            stops: stops.into(),
            dithering: true,
        }
    }

    pub fn angle_degrees(center: Vec2, angle_degrees: f32, stops: impl Into<Vec<GradientStop>>) -> Self {
        Self::new(center, angle_degrees.to_radians(), stops)
    }

    pub fn with_dithering(mut self, enabled: bool) -> Self {
        self.dithering = enabled;
        self
    }

    pub fn normalized_stops(&self) -> Vec<GradientStop> {
        normalize_stops(&self.stops)
    }
}

#[derive(Component, Clone, Debug, PartialEq)]
pub enum Paint {
    Solid(Color),
    Shimmer(super::Shimmer),
    LinearGradient(LinearGradient),
    RadialGradient(RadialGradient),
    AngularGradient(AngularGradient),
}

impl Paint {
    pub fn solid(color: Color) -> Self {
        Self::Solid(color)
    }

    pub fn linear(gradient: LinearGradient) -> Self {
        Self::LinearGradient(gradient)
    }

    pub fn radial(gradient: RadialGradient) -> Self {
        Self::RadialGradient(gradient)
    }

    pub fn angular(gradient: AngularGradient) -> Self {
        Self::AngularGradient(gradient)
    }

    /// Converts authored UI color into linear RGBA used by the shader.
    pub fn to_linear_rgba(&self) -> Option<LinearRgba> {
        match self {
            Self::Solid(color) => Some(color.to_linear()),
            Self::Shimmer(_) | Self::LinearGradient(_) | Self::RadialGradient(_) | Self::AngularGradient(_) => None,
        }
    }
}

impl Default for Paint {
    fn default() -> Self {
        Self::Solid(Color::WHITE)
    }
}

impl From<Color> for Paint {
    fn from(value: Color) -> Self {
        Self::Solid(value)
    }
}

pub fn normalize_stops(stops: &[GradientStop]) -> Vec<GradientStop> {
    let mut out: Vec<GradientStop> = stops
        .iter()
        .map(|stop| GradientStop {
            position: sanitize_stop_position(stop.position),
            color: stop.color,
        })
        .collect();

    out.sort_by(|a, b| a.position.total_cmp(&b.position));

    let mut deduped: Vec<GradientStop> = Vec::with_capacity(out.len());
    for stop in out {
        if let Some(last) = deduped.last_mut()
            && (stop.position - last.position).abs() <= 1e-6
        {
            *last = stop;
            continue;
        }

        deduped.push(stop);
    }

    deduped
}

fn sanitize_stop_position(position: f32) -> f32 {
    if !position.is_finite() {
        return 0.0;
    }

    position.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::{
        AngularGradient, GradientStop, LinearGradient, RadialGradient, normalize_stops,
    };
    use bevy::prelude::*;

    fn stop(position: f32, color: Color) -> GradientStop {
        GradientStop::new(position, color)
    }

    #[test]
    fn normalize_stops_sorts_and_clamps() {
        let stops = vec![
            stop(1.4, Color::srgb(0.0, 1.0, 0.0)),
            stop(-0.5, Color::srgb(1.0, 0.0, 0.0)),
            stop(0.5, Color::srgb(0.0, 0.0, 1.0)),
        ];

        let out = normalize_stops(&stops);
        assert_eq!(out.len(), 3);
        assert_eq!(out[0].position, 0.0);
        assert_eq!(out[1].position, 0.5);
        assert_eq!(out[2].position, 1.0);
    }

    #[test]
    fn normalize_stops_deduplicates_equal_positions_last_wins() {
        let red = Color::srgb(1.0, 0.0, 0.0);
        let blue = Color::srgb(0.0, 0.0, 1.0);
        let out = normalize_stops(&[stop(0.5, red), stop(0.5, blue)]);

        assert_eq!(out.len(), 1);
        assert_eq!(out[0].position, 0.5);
        assert_eq!(out[0].color.to_linear(), blue.to_linear());
    }

    #[test]
    fn normalize_stops_sanitizes_nan_and_infinity() {
        let out = normalize_stops(&[
            stop(f32::NAN, Color::WHITE),
            stop(2.0, Color::BLACK),
            stop(f32::NEG_INFINITY, Color::NONE),
        ]);

        assert_eq!(out[0].position, 0.0);
        assert_eq!(out[1].position, 1.0);
    }

    #[test]
    fn linear_convenience_constructors_create_expected_axes() {
        let h = LinearGradient::horizontal(vec![stop(0.0, Color::WHITE), stop(1.0, Color::BLACK)]);
        assert_eq!(h.start, Vec2::new(0.0, 0.5));
        assert_eq!(h.end, Vec2::new(1.0, 0.5));

        let v = LinearGradient::vertical(vec![stop(0.0, Color::WHITE), stop(1.0, Color::BLACK)]);
        assert_eq!(v.start, Vec2::new(0.5, 0.0));
        assert_eq!(v.end, Vec2::new(0.5, 1.0));
    }

    #[test]
    fn linear_angle_constructor_maps_zero_degrees_to_rightward() {
        let g = LinearGradient::angle_degrees(0.0, vec![stop(0.0, Color::WHITE), stop(1.0, Color::BLACK)]);
        assert!((g.start.x - 0.0).abs() < 1e-6);
        assert!((g.end.x - 1.0).abs() < 1e-6);
        assert!((g.start.y - 0.5).abs() < 1e-6);
        assert!((g.end.y - 0.5).abs() < 1e-6);
    }

    #[test]
    fn radial_and_angular_constructors_preserve_parameters() {
        let r = RadialGradient::circular(
            Vec2::new(0.25, 0.5),
            0.75,
            vec![stop(0.0, Color::WHITE), stop(1.0, Color::BLACK)],
        );
        assert_eq!(r.center, Vec2::new(0.25, 0.5));
        assert_eq!(r.radius, Vec2::splat(0.75));

        let a = AngularGradient::angle_degrees(
            Vec2::new(0.5, 0.5),
            180.0,
            vec![stop(0.0, Color::WHITE), stop(1.0, Color::BLACK)],
        );
        assert!((a.angle_radians - std::f32::consts::PI).abs() < 1e-6);
    }

    #[test]
    fn normalize_stops_handles_empty_and_single_stop() {
        let empty = normalize_stops(&[]);
        assert!(empty.is_empty());

        let one = normalize_stops(&[stop(0.25, Color::WHITE)]);
        assert_eq!(one.len(), 1);
        assert_eq!(one[0].position, 0.25);
    }
}
