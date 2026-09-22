/// Built-in placeholder highlight effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaceholderEffect {
    /// No animation.
    None,

    /// A bright band travels across the placeholder.
    Shimmer,

    /// A stronger rhythmic brightness animation.
    Pulse,

    /// The entire placeholder gently breathes.
    Fade,

    /// A broad soft wave moves through the placeholder.
    LightReveal,

    /// A circular/radial wave expands outward.
    CircularReveal,
}

impl PlaceholderEffect {
    pub fn shader_value(self) -> f32 {
        match self {
            Self::None => 0.0,
            Self::Shimmer => 1.0,
            Self::Pulse => 2.0,
            Self::Fade => 3.0,
            Self::LightReveal => 4.0,
            Self::CircularReveal => 5.0,
        }
    }
}

impl Default for PlaceholderEffect {
    fn default() -> Self {
        Self::Shimmer
    }
}

/// Represents the calculated visual state of an effect.
///
/// Keeping this separate makes it possible to eventually move
/// the rendering into a WGSL shader without changing the public
/// Placeholder API.
#[derive(Debug, Clone, Copy)]
pub struct PlaceholderVisual {
    pub brightness: f32,
    pub highlight_position: f32,
    pub highlight_alpha: f32,
}

impl Default for PlaceholderVisual {
    fn default() -> Self {
        Self {
            brightness: 1.0,
            highlight_position: 0.0,
            highlight_alpha: 0.0,
        }
    }
}

pub fn calculate_effect(effect: PlaceholderEffect, time: f32) -> PlaceholderVisual {
    match effect {
        PlaceholderEffect::None => PlaceholderVisual {
            brightness: 1.0,
            highlight_position: 0.0,
            highlight_alpha: 0.0,
        },

        PlaceholderEffect::Shimmer => {
            let progress = time.fract();

            PlaceholderVisual {
                brightness: 1.0,
                highlight_position: progress,
                highlight_alpha: 1.0,
            }
        }

        PlaceholderEffect::Fade => {
            let wave = (time.sin() + 1.0) * 0.5;

            PlaceholderVisual {
                brightness: 0.85 + wave * 0.15,
                highlight_position: 0.0,
                highlight_alpha: wave,
            }
        }

        PlaceholderEffect::Pulse => {
            let wave = (time.sin() + 1.0) * 0.5;

            PlaceholderVisual {
                brightness: 0.75 + wave * 0.25,
                highlight_position: 0.0,
                highlight_alpha: wave,
            }
        }

        PlaceholderEffect::LightReveal => {
            let progress = time.fract();

            PlaceholderVisual {
                brightness: 1.0,
                highlight_position: progress,
                highlight_alpha: 0.7,
            }
        }

        PlaceholderEffect::CircularReveal => {
            let progress = time.fract();

            PlaceholderVisual {
                brightness: 1.0,
                highlight_position: progress,
                highlight_alpha: 0.8,
            }
        }
    }
}
