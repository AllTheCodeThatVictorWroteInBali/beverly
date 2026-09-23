use bevy::prelude::*;
use std::time::Duration;

use crate::rendering::{
    FocusRing,
    FocusRingLayer,
    FocusRingPlacement,
    GradientStop,
    LinearGradient,
    OuterGlow,
    OuterShadow,
    Paint,
    ShadowFalloff,
};
use crate::animation::animation::{Easing, MotionClass, Transition};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug)]
pub struct ThemeColors {
    pub background: Color,
    pub surface: Color,
    pub surface_elevated: Color,
    /// Background for a hovered interactive surface (nav items, list rows, options).
    pub surface_hover: Color,
    /// Background for a selected/active interactive surface.
    pub surface_selected: Color,
    /// Neutral loading silhouette, distinct from the surrounding surface.
    pub skeleton_base: Color,
    /// Moving highlight for native skeleton shimmer paint.
    pub skeleton_highlight: Color,
    pub text: Color,
    pub text_muted: Color,
    pub text_disabled: Color,
    pub primary: Color,
    pub primary_hover: Color,
    pub primary_active: Color,
    pub secondary: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
    /// Fixed light neutral for `light`-styled surfaces (e.g. `ButtonColor::Light`).
    /// Deliberately does not adapt between light/dark theme modes.
    pub light_surface: Color,
    /// Fixed dark neutral for `dark`-styled surfaces (e.g. `ButtonColor::Dark`).
    /// Deliberately does not adapt between light/dark theme modes.
    pub dark_surface: Color,
    pub border: Color,
    pub border_strong: Color,
    pub focus: Color,
}

impl ThemeColors {
    pub fn surface_paint(self) -> Paint {
        Paint::linear(LinearGradient::vertical(vec![
            GradientStop::new(0.0, self.surface),
            GradientStop::new(1.0, self.surface_elevated),
        ]))
    }

    pub fn surface_elevated_paint(self) -> Paint {
        Paint::linear(LinearGradient::vertical(vec![
            GradientStop::new(0.0, self.surface_elevated),
            GradientStop::new(1.0, self.background),
        ]))
    }

    pub fn overlay_paint(self) -> Paint {
        Paint::solid(Color::srgba(0.0, 0.0, 0.0, if self.is_dark() { 0.55 } else { 0.45 }))
    }

    pub fn accent_paint(self) -> Paint {
        Paint::linear(LinearGradient::angle_degrees(
            20.0,
            vec![
                GradientStop::new(0.0, self.primary_hover),
                GradientStop::new(0.5, self.primary),
                GradientStop::new(1.0, self.primary_active),
            ],
        ))
    }

    pub fn border_paint(self) -> Paint {
        Paint::solid(self.border)
    }

    fn is_dark(self) -> bool {
        self.background.to_linear().luminance() < 0.25
    }

    pub fn default_outer_shadow(self, shadows: ThemeShadows) -> OuterShadow {
        let color = if self.is_dark() {
            Color::srgba(0.0, 0.0, 0.0, 1.0)
        } else {
            Color::BLACK
        };

        OuterShadow::small(color).with_opacity(shadows.medium_alpha)
    }

    pub fn focus_outer_glow(self) -> OuterGlow {
        OuterGlow::new(self.focus)
            .with_blur(16.0)
            .with_spread(2.0)
            .with_opacity(if self.is_dark() { 0.28 } else { 0.22 })
            .with_falloff(ShadowFalloff::Gaussian)
    }

    pub fn focus_ring(
        self,
        request: FocusStyleRequest,
        context: SurfaceContext,
        a11y: AccessibilityVisualPolicy,
    ) -> Option<FocusRing> {
        if !request.visible || matches!(a11y.focus_visibility, FocusVisibilityPolicy::Hidden) {
            return None;
        }

        let mut primary_color = match request.intent {
            FocusIntent::Default => self.focus,
            FocusIntent::Selected => self.info,
            FocusIntent::Invalid => self.error,
        };

        if matches!(a11y.contrast, AccessibilityContrastMode::High) {
            primary_color = if context.dark_background {
                Color::WHITE
            } else {
                Color::BLACK
            };
        }

        let base_width = if matches!(a11y.contrast, AccessibilityContrastMode::High) {
            3.0
        } else {
            2.0
        };

        let mut ring = FocusRing::outside(base_width, 2.0, Paint::solid(primary_color))
            .with_placement(FocusRingPlacement::Outside);

        let wants_dual = matches!(a11y.contrast, AccessibilityContrastMode::High)
            || matches!(context.tone, SurfaceTone::Glass | SurfaceTone::Accent | SurfaceTone::Danger);
        if wants_dual {
            let secondary_color = if primary_color.to_linear().luminance() > 0.5 {
                Color::BLACK
            } else {
                Color::WHITE
            };
            ring = ring.with_secondary(
                FocusRingLayer::new(base_width + 1.0, 0.0, Paint::solid(secondary_color))
                    .with_opacity(0.95),
            );
        }

        if !a11y.reduced_effects && !matches!(a11y.contrast, AccessibilityContrastMode::High) {
            ring = ring.with_glow(self.focus_outer_glow());
        }

        Some(ring)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ThemeTypography {
    pub font_size_body: f32,
    pub font_size_title: f32,
    pub font_size_caption: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct ThemeSpacing {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct ThemeRadius {
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub pill: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct ThemeBorders {
    pub thin: f32,
    pub strong: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct ThemeShadows {
    pub low_alpha: f32,
    pub medium_alpha: f32,
    pub high_alpha: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct ThemeVisualEffects {
    /// Decorative surface noise defaults to off unless explicitly enabled.
    pub default_noise_strength: f32,
    /// Accessibility switch for reducing decorative effects.
    pub reduced_effects: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccessibilityContrastMode {
    Normal,
    High,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusVisibilityPolicy {
    Always,
    KeyboardOnly,
    Programmatic,
    Hidden,
}

#[derive(Clone, Copy, Debug)]
pub struct AccessibilityVisualPolicy {
    pub contrast: AccessibilityContrastMode,
    pub reduced_motion: bool,
    pub reduced_effects: bool,
    pub reduced_transparency: bool,
    pub focus_visibility: FocusVisibilityPolicy,
}

impl Default for AccessibilityVisualPolicy {
    fn default() -> Self {
        Self {
            contrast: AccessibilityContrastMode::Normal,
            reduced_motion: false,
            reduced_effects: false,
            reduced_transparency: false,
            focus_visibility: FocusVisibilityPolicy::KeyboardOnly,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ThemeTransitions {
    pub interaction: Transition,
    pub focus: Transition,
    pub modal: Transition,
    pub tooltip: Transition,
}

impl Default for ThemeTransitions {
    fn default() -> Self {
        Self {
            interaction: Transition::new(Duration::from_millis(120), Easing::EaseOut)
                .with_motion_class(MotionClass::Decorative),
            focus: Transition::new(Duration::from_millis(90), Easing::Smooth)
                .with_motion_class(MotionClass::Semantic),
            modal: Transition::new(Duration::from_millis(180), Easing::EaseOut)
                .with_motion_class(MotionClass::Decorative),
            tooltip: Transition::new(Duration::from_millis(140), Easing::EaseOut)
                .with_motion_class(MotionClass::Decorative),
        }
    }
}

#[derive(Resource, Clone, Copy, Debug)]
pub struct AccessibilityVisualPolicyResource {
    pub current: AccessibilityVisualPolicy,
}

impl Default for AccessibilityVisualPolicyResource {
    fn default() -> Self {
        let contrast = match std::env::var("UI_HIGH_CONTRAST").ok().as_deref() {
            Some("1") | Some("true") | Some("TRUE") | Some("yes") | Some("on") => {
                AccessibilityContrastMode::High
            }
            _ => AccessibilityContrastMode::Normal,
        };
        let reduced_motion = matches!(
            std::env::var("UI_REDUCED_MOTION").ok().as_deref(),
            Some("1") | Some("true") | Some("TRUE") | Some("yes") | Some("on")
        );
        let reduced_effects = matches!(
            std::env::var("UI_REDUCED_EFFECTS").ok().as_deref(),
            Some("1") | Some("true") | Some("TRUE") | Some("yes") | Some("on")
        );
        let reduced_transparency = matches!(
            std::env::var("UI_REDUCED_TRANSPARENCY").ok().as_deref(),
            Some("1") | Some("true") | Some("TRUE") | Some("yes") | Some("on")
        );
        let focus_visibility = match std::env::var("UI_FOCUS_VISIBILITY").ok().as_deref() {
            Some("always") => FocusVisibilityPolicy::Always,
            Some("programmatic") => FocusVisibilityPolicy::Programmatic,
            Some("hidden") => FocusVisibilityPolicy::Hidden,
            _ => FocusVisibilityPolicy::KeyboardOnly,
        };

        Self {
            current: AccessibilityVisualPolicy {
                contrast,
                reduced_motion,
                reduced_effects,
                reduced_transparency,
                focus_visibility,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceTone {
    Neutral,
    Accent,
    Danger,
    Success,
    Warning,
    Info,
    Glass,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SurfaceContext {
    pub tone: SurfaceTone,
    pub dark_background: bool,
}

impl Default for SurfaceContext {
    fn default() -> Self {
        Self {
            tone: SurfaceTone::Neutral,
            dark_background: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusIntent {
    Default,
    Selected,
    Invalid,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FocusStyleRequest {
    pub intent: FocusIntent,
    pub visible: bool,
}

impl Default for FocusStyleRequest {
    fn default() -> Self {
        Self {
            intent: FocusIntent::Default,
            visible: true,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub mode: ThemeMode,
    pub colors: ThemeColors,
    pub typography: ThemeTypography,
    pub spacing: ThemeSpacing,
    pub radius: ThemeRadius,
    pub borders: ThemeBorders,
    pub shadows: ThemeShadows,
    pub visual_effects: ThemeVisualEffects,
    pub transitions: ThemeTransitions,
}

#[derive(Resource, Clone, Copy, Debug)]
pub struct ThemeResource {
    pub current: Theme,
}

impl Default for ThemeResource {
    fn default() -> Self {
        Self {
            current: light_theme(),
        }
    }
}

#[derive(Message, Debug, Clone, Copy)]
pub struct ThemeChanged {
    pub mode: ThemeMode,
}

pub fn light_theme() -> Theme {
    Theme {
        mode: ThemeMode::Light,
        colors: ThemeColors {
            background: Color::srgb(0.973, 0.980, 0.988),
            surface: Color::srgb(1.0, 1.0, 1.0),
            surface_elevated: Color::srgb(1.0, 1.0, 1.0),
            surface_hover: Color::srgb(0.961, 0.968, 0.980),
            surface_selected: Color::srgb(0.925, 0.941, 0.965),
            skeleton_base: Color::srgb(0.855, 0.879, 0.914),
            skeleton_highlight: Color::srgb(0.961, 0.973, 0.988),
            text: Color::srgb(0.059, 0.090, 0.165),
            text_muted: Color::srgb(0.392, 0.455, 0.545),
            text_disabled: Color::srgb(0.580, 0.639, 0.722),
            primary: Color::srgb(0.145, 0.388, 0.922),
            primary_hover: Color::srgb(0.114, 0.306, 0.847),
            primary_active: Color::srgb(0.118, 0.251, 0.686),
            secondary: Color::srgb(0.945, 0.961, 0.976),
            success: Color::srgb(0.086, 0.639, 0.290),
            warning: Color::srgb(0.851, 0.467, 0.024),
            error: Color::srgb(0.863, 0.149, 0.149),
            info: Color::srgb(0.027, 0.518, 0.761),
            light_surface: Color::srgb(0.973, 0.976, 0.980),
            dark_surface: Color::srgb(0.129, 0.145, 0.161),
            border: Color::srgb(0.886, 0.910, 0.941),
            border_strong: Color::srgb(0.796, 0.835, 0.882),
            focus: Color::srgb(0.231, 0.510, 0.965),
        },
        typography: ThemeTypography {
            font_size_body: 16.0,
            font_size_title: 24.0,
            font_size_caption: 13.0,
        },
        spacing: ThemeSpacing {
            xs: 4.0,
            sm: 8.0,
            md: 12.0,
            lg: 16.0,
            xl: 24.0,
        },
        radius: ThemeRadius {
            sm: 4.0,
            md: 6.0,
            lg: 8.0,
            pill: 999.0,
        },
        borders: ThemeBorders {
            thin: 1.0,
            strong: 2.0,
        },
        shadows: ThemeShadows {
            low_alpha: 0.06,
            medium_alpha: 0.12,
            high_alpha: 0.18,
        },
        visual_effects: ThemeVisualEffects {
            default_noise_strength: 0.0,
            reduced_effects: false,
        },
        transitions: ThemeTransitions::default(),
    }
}

pub fn dark_theme() -> Theme {
    Theme {
        mode: ThemeMode::Dark,
        colors: ThemeColors {
            // Neutral near-black/charcoal foundation (not navy) with a clear,
            // restrained surface hierarchy: background < surface < elevated
            // < hover < selected.
            background: Color::srgb(0.039, 0.039, 0.047),
            surface: Color::srgb(0.075, 0.075, 0.086),
            surface_elevated: Color::srgb(0.110, 0.110, 0.125),
            surface_hover: Color::srgb(0.133, 0.133, 0.149),
            surface_selected: Color::srgb(0.153, 0.157, 0.180),
            skeleton_base: Color::srgb(0.200, 0.204, 0.231),
            skeleton_highlight: Color::srgb(0.345, 0.353, 0.392),
            text: Color::srgb(0.973, 0.980, 0.988),
            text_muted: Color::srgb(0.580, 0.639, 0.722),
            text_disabled: Color::srgb(0.278, 0.333, 0.412),
            primary: Color::srgb(0.382, 0.618, 0.961),
            primary_hover: Color::srgb(0.489, 0.706, 0.984),
            primary_active: Color::srgb(0.294, 0.467, 0.812),
            secondary: Color::srgb(0.129, 0.129, 0.145),
            success: Color::srgb(0.167, 0.758, 0.467),
            warning: Color::srgb(0.969, 0.682, 0.200),
            error: Color::srgb(0.933, 0.333, 0.333),
            info: Color::srgb(0.118, 0.678, 0.918),
            light_surface: Color::srgb(0.973, 0.976, 0.980),
            dark_surface: Color::srgb(0.129, 0.145, 0.161),
            // Subtle, low-contrast hairlines by default; `border_strong` is
            // reserved for focused/selected/validated emphasis.
            border: Color::srgb(0.165, 0.165, 0.188),
            border_strong: Color::srgb(0.290, 0.290, 0.322),
            focus: Color::srgb(0.540, 0.744, 0.989),
        },
        typography: ThemeTypography {
            font_size_body: 16.0,
            font_size_title: 24.0,
            font_size_caption: 13.0,
        },
        spacing: ThemeSpacing {
            xs: 4.0,
            sm: 8.0,
            md: 12.0,
            lg: 16.0,
            xl: 24.0,
        },
        radius: ThemeRadius {
            sm: 4.0,
            md: 6.0,
            lg: 8.0,
            pill: 999.0,
        },
        borders: ThemeBorders {
            thin: 1.0,
            strong: 2.0,
        },
        shadows: ThemeShadows {
            low_alpha: 0.16,
            medium_alpha: 0.22,
            high_alpha: 0.30,
        },
        visual_effects: ThemeVisualEffects {
            default_noise_strength: 0.0,
            reduced_effects: false,
        },
        transitions: ThemeTransitions::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dark_theme_keeps_surface_hierarchy_and_readable_muted_text() {
        let colors = dark_theme().colors;
        let background_luminance = colors.background.to_linear().luminance();
        let surface_luminance = colors.surface.to_linear().luminance();
        let elevated_luminance = colors.surface_elevated.to_linear().luminance();
        let muted_luminance = colors.text_muted.to_linear().luminance();

        assert!(surface_luminance > background_luminance);
        assert!(elevated_luminance > surface_luminance);
        assert!(muted_luminance > 0.35, "muted text should stay readable against the dark background");
        assert!(colors.border.to_linear().luminance() < 0.45);
    }
}

fn apply_theme_change(mut events: MessageReader<ThemeChanged>, mut theme: ResMut<ThemeResource>) {
    for event in events.read() {
        let next = match event.mode {
            ThemeMode::Light => light_theme(),
            ThemeMode::Dark => dark_theme(),
        };

        if next.mode != theme.current.mode {
            theme.current = next;
        }
    }
}

pub struct ThemePlugin;

impl Plugin for ThemePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ThemeResource>()
            .init_resource::<AccessibilityVisualPolicyResource>()
            .add_message::<ThemeChanged>()
            .add_systems(Update, apply_theme_change);
    }
}
