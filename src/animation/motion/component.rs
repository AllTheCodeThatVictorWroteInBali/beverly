use bevy::prelude::*;

use crate::rendering::{Paint, Surface};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UiMotionPreset {
    Fade,
    FadeUp { distance: f32 },
    FadeDown { distance: f32 },
    Pop,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiMotionSpec {
    pub preset: UiMotionPreset,
    pub duration_secs: f32,
    pub delay_secs: f32,
}

impl UiMotionSpec {
    pub fn fade(duration_secs: f32) -> Self {
        Self {
            preset: UiMotionPreset::Fade,
            duration_secs,
            delay_secs: 0.0,
        }
    }

    pub fn fade_up(duration_secs: f32, distance: f32) -> Self {
        Self {
            preset: UiMotionPreset::FadeUp { distance },
            duration_secs,
            delay_secs: 0.0,
        }
    }

    pub fn fade_down(duration_secs: f32, distance: f32) -> Self {
        Self {
            preset: UiMotionPreset::FadeDown { distance },
            duration_secs,
            delay_secs: 0.0,
        }
    }

    pub fn pop(duration_secs: f32) -> Self {
        Self {
            preset: UiMotionPreset::Pop,
            duration_secs,
            delay_secs: 0.0,
        }
    }

    pub fn delay(mut self, delay_secs: f32) -> Self {
        self.delay_secs = delay_secs.max(0.0);
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiMotionPhase {
    Entering,
    Idle,
    Exiting,
}

#[derive(Component, Clone, Debug)]
pub struct UiMotion {
    pub enter: Option<UiMotionSpec>,
    pub exit: Option<UiMotionSpec>,
    pub phase: UiMotionPhase,
    elapsed_secs: f32,
}

impl UiMotion {
    pub fn new(enter: Option<UiMotionSpec>, exit: Option<UiMotionSpec>) -> Self {
        Self {
            enter,
            exit,
            phase: if enter.is_some() {
                UiMotionPhase::Entering
            } else {
                UiMotionPhase::Idle
            },
            elapsed_secs: 0.0,
        }
    }

    pub fn fade() -> Self {
        Self::new(
            Some(UiMotionSpec::fade(0.22)),
            Some(UiMotionSpec::fade(0.16)),
        )
    }

    pub fn fade_up() -> Self {
        Self::new(
            Some(UiMotionSpec::fade_up(0.24, 10.0)),
            Some(UiMotionSpec::fade_down(0.16, 8.0)),
        )
    }

    pub fn pop() -> Self {
        Self::new(
            Some(UiMotionSpec::pop(0.20)),
            Some(UiMotionSpec::fade(0.14)),
        )
    }

    pub fn with_enter(mut self, enter: UiMotionSpec) -> Self {
        self.enter = Some(enter);
        if self.phase == UiMotionPhase::Idle {
            self.phase = UiMotionPhase::Entering;
        }
        self
    }

    pub fn with_exit(mut self, exit: UiMotionSpec) -> Self {
        self.exit = Some(exit);
        self
    }

    pub fn restart_enter(&mut self) {
        self.elapsed_secs = 0.0;
        self.phase = if self.enter.is_some() {
            UiMotionPhase::Entering
        } else {
            UiMotionPhase::Idle
        };
    }
}

#[derive(Component)]
pub struct UiMotionExitRequested;

#[derive(Component, Clone, Copy, Debug)]
struct UiMotionBase {
    top_px: f32,
    background_alpha: Option<f32>,
    text_alpha: Option<f32>,
}

pub struct UiMotionPlugin;

impl Plugin for UiMotionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (capture_motion_base, begin_exit_phase, tick_ui_motion).chain(),
        );
    }
}

fn capture_motion_base(
    mut commands: Commands,
    query: Query<
        (
            Entity,
            Option<&Node>,
            Option<&Surface>,
            Option<&TextColor>,
        ),
        (With<UiMotion>, Added<UiMotion>, Without<UiMotionBase>),
    >,
) {
    for (entity, node, background, text) in &query {
        let top_px = node
            .and_then(|n| match n.margin.top {
                Val::Px(v) => Some(v),
                _ => None,
            })
            .unwrap_or(0.0);

        let base = UiMotionBase {
            top_px,
            background_alpha: background.map(|b| match &b.fill {
                Paint::Solid(color) => color.alpha(),
                _ => 1.0,
            }),
            text_alpha: text.map(|t| t.0.alpha()),
        };

        commands.entity(entity).insert(base);
    }
}

fn begin_exit_phase(
    mut commands: Commands,
    mut query: Query<(Entity, &mut UiMotion), With<UiMotionExitRequested>>,
) {
    for (entity, mut motion) in &mut query {
        if motion.phase != UiMotionPhase::Exiting {
            motion.phase = UiMotionPhase::Exiting;
            motion.elapsed_secs = 0.0;
        }

        // Keep the marker only while we are transitioning out.
        if motion.exit.is_none() {
            commands.entity(entity).remove::<UiMotionExitRequested>();
        }
    }
}

fn tick_ui_motion(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &mut UiMotion,
        Option<&UiMotionBase>,
        Option<&mut Node>,
        Option<&mut Surface>,
        Option<&mut TextColor>,
        Option<&UiMotionExitRequested>,
    )>,
) {
    for (entity, mut motion, base, node, background, text, exit_requested) in &mut query {
        let spec = match motion.phase {
            UiMotionPhase::Entering => motion.enter,
            UiMotionPhase::Exiting => motion.exit,
            UiMotionPhase::Idle => None,
        };

        let Some(spec) = spec else {
            if motion.phase == UiMotionPhase::Exiting {
                commands
                    .entity(entity)
                    .despawn_related::<Children>()
                    .despawn();
            }
            continue;
        };

        motion.elapsed_secs += time.delta_secs();

        let elapsed_after_delay = (motion.elapsed_secs - spec.delay_secs).max(0.0);
        let duration = spec.duration_secs.max(0.001);
        let t = (elapsed_after_delay / duration).clamp(0.0, 1.0);

        let eased = match motion.phase {
            UiMotionPhase::Entering => ease_out_cubic(t),
            UiMotionPhase::Exiting => ease_in_cubic(t),
            UiMotionPhase::Idle => t,
        };

        let alpha_factor = match motion.phase {
            UiMotionPhase::Entering => eased,
            UiMotionPhase::Exiting => 1.0 - eased,
            UiMotionPhase::Idle => 1.0,
        };

        if let Some(base) = base {
            apply_alpha(background, base.background_alpha, alpha_factor);
            apply_text_alpha(text, base.text_alpha, alpha_factor);
            apply_offset(node, *base, spec.preset, motion.phase, eased);
        }

        if t >= 1.0 {
            match motion.phase {
                UiMotionPhase::Entering => {
                    motion.phase = UiMotionPhase::Idle;
                    motion.elapsed_secs = 0.0;
                }
                UiMotionPhase::Exiting => {
                    commands
                        .entity(entity)
                        .despawn_related::<Children>()
                        .despawn();
                }
                UiMotionPhase::Idle => {}
            }
        }

        if exit_requested.is_some() && motion.phase == UiMotionPhase::Idle {
            motion.phase = UiMotionPhase::Exiting;
            motion.elapsed_secs = 0.0;
        }
    }
}

fn apply_alpha(
    mut background: Option<Mut<Surface>>,
    base_alpha: Option<f32>,
    alpha_factor: f32,
) {
    let (Some(mut background), Some(base_alpha)) = (background.take(), base_alpha) else {
        return;
    };

    let alpha = (base_alpha * alpha_factor).clamp(0.0, 1.0);
    background.fill = match &background.fill {
        Paint::Solid(color) => Paint::solid(color.with_alpha(alpha)),
        other => other.clone(),
    };
}

fn apply_text_alpha(mut text: Option<Mut<TextColor>>, base_alpha: Option<f32>, alpha_factor: f32) {
    let (Some(mut text), Some(base_alpha)) = (text.take(), base_alpha) else {
        return;
    };

    let alpha = (base_alpha * alpha_factor).clamp(0.0, 1.0);
    text.0 = text.0.with_alpha(alpha);
}

fn apply_offset(
    mut node: Option<Mut<Node>>,
    base: UiMotionBase,
    preset: UiMotionPreset,
    phase: UiMotionPhase,
    eased: f32,
) {
    let Some(mut node) = node.take() else {
        return;
    };

    let distance = match preset {
        UiMotionPreset::Fade => 0.0,
        UiMotionPreset::FadeUp { distance } => distance,
        UiMotionPreset::FadeDown { distance } => -distance,
        UiMotionPreset::Pop => 6.0,
    };

    let offset = match phase {
        UiMotionPhase::Entering => (1.0 - eased) * distance,
        UiMotionPhase::Exiting => eased * distance,
        UiMotionPhase::Idle => 0.0,
    };

    node.margin.top = Val::Px(base.top_px + offset);
}

fn ease_out_cubic(t: f32) -> f32 {
    let u = 1.0 - t;
    1.0 - u * u * u
}

fn ease_in_cubic(t: f32) -> f32 {
    t * t * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restart_enter_rearms_completed_motion() {
        let mut motion = UiMotion::fade_up();
        motion.phase = UiMotionPhase::Idle;
        motion.elapsed_secs = 0.37;

        motion.restart_enter();

        assert_eq!(motion.phase, UiMotionPhase::Entering);
        assert_eq!(motion.elapsed_secs, 0.0);
    }
}
