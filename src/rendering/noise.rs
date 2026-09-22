const NOISE_MIN_SCALE: f32 = 1e-3;
const NOISE_MAX_SCALE: f32 = 4096.0;
const NOISE_MAX_SPEED: f32 = 16.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum NoiseKind {
    #[default]
    Grain,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum NoiseTarget {
    #[default]
    Surface,
    Fill,
    Border,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum NoiseSpace {
    #[default]
    Local,
}

/// Procedural appearance variation that can be composed with any `Surface`.
///
/// Semantics:
/// - Noise does not define geometry.
/// - Noise modulates appearance in linear color space.
/// - Noise does not modify alpha by default.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Noise {
    pub kind: NoiseKind,
    /// Grain size in local physical pixels. Larger values produce coarser noise.
    pub scale: f32,
    /// Modulation amplitude in [0, 1].
    pub strength: f32,
    /// Deterministic variation between surfaces.
    pub seed: f32,
    /// Enables smooth time-based animation when true.
    pub animated: bool,
    /// Animation speed multiplier in local noise space units per second.
    pub speed: f32,
    /// Coordinate space for sampling. Initial implementation supports local space.
    pub space: NoiseSpace,
    /// Region to modulate.
    pub target: NoiseTarget,
}

impl Noise {
    pub fn grain(scale: f32, strength: f32) -> Self {
        Self {
            scale,
            strength,
            ..Self::default()
        }
    }

    pub fn with_seed(mut self, seed: f32) -> Self {
        self.seed = seed;
        self
    }

    pub fn with_animated(mut self, animated: bool) -> Self {
        self.animated = animated;
        self
    }

    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    pub fn with_space(mut self, space: NoiseSpace) -> Self {
        self.space = space;
        self
    }

    pub fn with_target(mut self, target: NoiseTarget) -> Self {
        self.target = target;
        self
    }

    pub fn sanitized(self) -> Self {
        let scale = sanitize_finite(self.scale, 24.0)
            .abs()
            .clamp(NOISE_MIN_SCALE, NOISE_MAX_SCALE);
        let strength = sanitize_finite(self.strength, 0.0).clamp(0.0, 1.0);
        let seed = sanitize_finite(self.seed, 0.0);
        let speed = sanitize_finite(self.speed, 0.0)
            .abs()
            .clamp(0.0, NOISE_MAX_SPEED);

        Self {
            kind: self.kind,
            scale,
            strength,
            seed,
            animated: self.animated,
            speed,
            space: self.space,
            target: self.target,
        }
    }

    pub fn is_active(self) -> bool {
        let value = self.sanitized();
        value.strength > 1e-5
    }
}

impl Default for Noise {
    fn default() -> Self {
        Self {
            kind: NoiseKind::Grain,
            scale: 24.0,
            strength: 0.0,
            seed: 0.0,
            animated: false,
            speed: 0.0,
            space: NoiseSpace::Local,
            target: NoiseTarget::Surface,
        }
    }
}

fn sanitize_finite(value: f32, fallback: f32) -> f32 {
    if value.is_finite() { value } else { fallback }
}

#[cfg(test)]
mod tests {
    use super::{Noise, NoiseSpace, NoiseTarget};

    #[test]
    fn defaults_are_safe_and_off() {
        let noise = Noise::default();
        assert_eq!(noise.strength, 0.0);
        assert_eq!(noise.scale, 24.0);
        assert_eq!(noise.target, NoiseTarget::Surface);
        assert_eq!(noise.space, NoiseSpace::Local);
        assert!(!noise.is_active());
    }

    #[test]
    fn sanitizes_invalid_values() {
        let noise = Noise::grain(f32::NAN, -1.0)
            .with_seed(f32::INFINITY)
            .with_speed(f32::NEG_INFINITY)
            .sanitized();

        assert!(noise.scale >= 1e-3);
        assert_eq!(noise.strength, 0.0);
        assert!(noise.seed.is_finite());
        assert_eq!(noise.speed, 0.0);
    }

    #[test]
    fn clamps_extreme_values() {
        let noise = Noise::grain(100_000.0, 10.0)
            .with_speed(9999.0)
            .sanitized();

        assert_eq!(noise.scale, 4096.0);
        assert_eq!(noise.strength, 1.0);
        assert_eq!(noise.speed, 16.0);
    }

    #[test]
    fn active_when_strength_is_positive() {
        let noise = Noise::grain(16.0, 0.02);
        assert!(noise.is_active());
    }
}
