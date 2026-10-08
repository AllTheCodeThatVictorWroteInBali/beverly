//! Procedural animated paint. Time is Bevy's existing GPU globals clock.
use bevy::prelude::*;

use super::paint::{GradientStop, MAX_GRADIENT_STOPS};

/// An angular gradient that spins around the surface on the GPU, so it needs no
/// per-frame system. Stops are positions around the full turn; the motion curve
/// is fixed and only the colors are authored.
///
/// The default is a mostly-neutral line with a bright blue-white highlight that
/// chases around it. Pass any stops to replace it:
///
/// ```
/// # use beverly::prelude::*;
/// # use bevy::prelude::*;
/// let surface = Surface::rounded_rect_fill(8.0, Color::WHITE).animated_border(
///     SpinningGradient::new([
///         GradientStop::new(0.0, Color::srgb(0.9, 0.2, 0.4)),
///         GradientStop::new(0.5, Color::srgb(1.0, 0.8, 0.2)),
///         GradientStop::new(1.0, Color::srgb(0.9, 0.2, 0.4)),
///     ]),
/// );
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct SpinningGradient {
    /// At most [`MAX_GRADIENT_STOPS`]; extra stops are ignored.
    pub stops: Vec<GradientStop>,
    pub enabled: bool,
}

impl SpinningGradient {
    pub fn new(stops: impl Into<Vec<GradientStop>>) -> Self {
        Self {
            stops: stops.into(),
            enabled: true,
        }
    }

    pub fn normalized_stops(&self) -> Vec<GradientStop> {
        let mut stops = super::paint::normalize_stops(&self.stops);
        stops.truncate(MAX_GRADIENT_STOPS);
        stops
    }
}

impl Default for SpinningGradient {
    fn default() -> Self {
        let gray = Color::srgba(0.29, 0.34, 0.46, 0.48);
        let blue = Color::srgb(0.231, 0.510, 0.965);
        Self::new([
            GradientStop::new(0.60, gray),
            GradientStop::new(0.76, blue),
            GradientStop::new(0.86, Color::srgb(0.96, 0.98, 1.0)),
            GradientStop::new(0.94, blue),
            GradientStop::new(1.0, gray),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_gradient_fits_the_gpu_stop_budget() {
        let stops = SpinningGradient::default().normalized_stops();
        assert_eq!(stops.len(), MAX_GRADIENT_STOPS);
        assert_eq!(stops.first().unwrap().color, stops.last().unwrap().color);
    }

    #[test]
    fn custom_stops_are_sorted_and_capped() {
        let stops = (0..8)
            .rev()
            .map(|index| GradientStop::new(index as f32 / 7.0, Color::WHITE))
            .collect::<Vec<_>>();
        let normalized = SpinningGradient::new(stops).normalized_stops();
        assert_eq!(normalized.len(), MAX_GRADIENT_STOPS);
        assert!(
            normalized
                .windows(2)
                .all(|w| w[0].position <= w[1].position)
        );
    }
}
