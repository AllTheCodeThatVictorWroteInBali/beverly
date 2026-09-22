//! Procedural animated paint. Time is Bevy's existing GPU globals clock.
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ShimmerDirection {
    #[default]
    LeftToRight,
    RightToLeft,
}

/// A narrow, soft highlight in normalized local coordinates. Geometry is supplied
/// by the enclosing Surface, never by the paint. Phase is measured in cycles.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shimmer {
    pub base_color: Color,
    pub highlight_color: Color,
    pub duration: f32,
    pub direction: ShimmerDirection,
    pub phase: f32,
    /// Full width of the highlight band relative to the region's width.
    pub width: f32,
    /// Fraction of the half-band used for each smooth edge.
    pub softness: f32,
    pub intensity: f32,
    pub enabled: bool,
}

impl Shimmer {
    pub fn new(base_color: Color, highlight_color: Color) -> Self {
        Self { base_color, highlight_color, ..default() }
    }

    pub fn sanitized(mut self) -> Self {
        fn finite(value: f32, fallback: f32, min: f32, max: f32) -> f32 {
            if value.is_finite() { value.clamp(min, max) } else { fallback }
        }
        self.duration = finite(self.duration, 1.5, 0.1, 3600.0);
        self.phase = if self.phase.is_finite() { self.phase.rem_euclid(1.0) } else { 0.0 };
        self.width = finite(self.width, 0.22, 0.01, 0.8);
        self.softness = finite(self.softness, 0.8, 0.05, 1.0);
        self.intensity = finite(self.intensity, 0.65, 0.0, 1.0);
        self
    }
}

impl Default for Shimmer {
    fn default() -> Self {
        Self {
            // Neutral rendering defaults; UI Skeleton always resolves theme roles.
            base_color: Color::NONE, highlight_color: Color::NONE,
            duration: 1.5, direction: ShimmerDirection::LeftToRight,
            phase: 0.0, width: 0.22, softness: 0.8, intensity: 0.65, enabled: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shimmer_defaults_and_constructor_preserve_authored_colors() {
        let base = Color::linear_rgba(0.2, 0.4, 0.6, 0.25);
        let highlight = Color::linear_rgba(0.8, 0.6, 0.4, 0.75);
        let shimmer = Shimmer::new(base, highlight);
        assert_eq!(shimmer, Shimmer {
            base_color: base, highlight_color: highlight, duration: 1.5,
            direction: ShimmerDirection::LeftToRight, phase: 0.0,
            width: 0.22, softness: 0.8, intensity: 0.65, enabled: true,
        });
        assert_eq!(shimmer.sanitized(), shimmer);
        assert_eq!(Shimmer::default().base_color, Color::NONE);
        assert_eq!(Shimmer::default().highlight_color, Color::NONE);
    }

    #[test]
    fn shimmer_nonfinite_inputs_use_defaults_not_finite_clamp_endpoints() {
        for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let shimmer = Shimmer {
                duration: invalid, phase: invalid, width: invalid,
                softness: invalid, intensity: invalid,
                direction: ShimmerDirection::RightToLeft, enabled: false,
                ..default()
            }.sanitized();
            assert_eq!(shimmer, Shimmer {
                direction: ShimmerDirection::RightToLeft, enabled: false,
                ..default()
            });
        }
    }

    #[test]
    fn shimmer_finite_extremes_clamp_without_degenerate_smoothstep_edges() {
        for (input, duration, width, softness, intensity) in [
            (f32::MIN, 0.1, 0.01, 0.05, 0.0),
            (-1.0, 0.1, 0.01, 0.05, 0.0),
            (-0.0, 0.1, 0.01, 0.05, 0.0),
            (f32::from_bits(1), 0.1, 0.01, 0.05, f32::from_bits(1)),
            (0.5, 0.5, 0.5, 0.5, 0.5),
            (1.0, 1.0, 0.8, 1.0, 1.0),
            (f32::MAX, 3600.0, 0.8, 1.0, 1.0),
        ] {
            let shimmer = Shimmer {
                duration: input, width: input, softness: input, intensity: input,
                ..default()
            }.sanitized();
            assert_eq!((shimmer.duration, shimmer.width, shimmer.softness, shimmer.intensity),
                (duration, width, softness, intensity), "input={input}");
            let half_width = shimmer.width * 0.5;
            let edge_start = half_width * (1.0 - shimmer.softness);
            assert!(edge_start.is_finite() && edge_start >= 0.0 && edge_start < half_width);
            assert_eq!(shimmer.sanitized(), shimmer);
        }
    }

    #[test]
    fn shimmer_phase_is_a_cycle_offset_including_negative_and_extreme_inputs() {
        for (input, expected) in [
            (-2.25, 0.75), (-1.0, 0.0), (0.0, 0.0), (0.25, 0.25),
            (1.0, 0.0), (3.25, 0.25), (f32::MIN, 0.0), (f32::MAX, 0.0),
        ] {
            let shimmer = Shimmer { phase: input, ..default() }.sanitized();
            assert_eq!(shimmer.phase, expected, "input={input}");
        }
        // f32 rem_euclid may round a tiny negative remainder up to 1.0.
        // The shader's fract still maps that equivalent cycle offset to zero.
        let phase = Shimmer { phase: -f32::from_bits(1), ..default() }.sanitized().phase;
        assert!(phase.is_finite() && (0.0..=1.0).contains(&phase));
        assert_eq!(phase.fract(), 0.0);
    }
}